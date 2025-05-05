use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::marker::PhantomData;


use tauri::{AppHandle, Event, Listener, Runtime, WebviewWindow, WindowEvent};
use pin_project::pin_project;
use crate::models::DialogResult;


pub struct AwaitEvent<R: Runtime> {
    pub state: Arc<Mutex<StateSimple>>,
    unlisten_id: u32,
    app_handle: AppHandle<R>,
}

pub struct State<R: Runtime> {
    completed: bool,
    payload: Option<DialogResult>,
    waker: Option<Waker>,
    unlisten_id: Option<u32>,
    app_handle: AppHandle<R>,
}

impl<R: Runtime> AwaitEvent<R> {
    pub fn new(app: AppHandle<R>, event_name: &str) -> Self {
        let state = Arc::new(Mutex::new(StateSimple {
            completed: false,
            payload: None,
            waker: None,
        }));
        let state_clone = state.clone();
        let app_clone = app.clone();
        // let state_clone_inner = state.clone();

        let listener_id = app.listen(event_name, move |event: Event| {
            let mut state_unwrapped_inner = state_clone.lock().unwrap();
            state_unwrapped_inner.completed = true;
            let result: DialogResult = serde_json::from_str(event.payload())
                .unwrap_or_else(|e| {
                    eprintln!("Error deserializing DialogResult: {}, Raw: {:?}", e, state_unwrapped_inner.payload);
                    DialogResult::Closed
                });
            state_unwrapped_inner.payload = Some(result);

            if let Some(waker) = state_unwrapped_inner.waker.take() {
                waker.wake();
            }

        });

        app_clone.unlisten(listener_id);

        AwaitEvent::<R> { state, unlisten_id: listener_id, app_handle: app }
    }

}

impl<R: Runtime> std::future::Future for AwaitEvent<R> {
    type Output = Option<DialogResult>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.completed {
            Poll::Ready(state.payload.take())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<R: Runtime> Drop for AwaitEvent<R> {
    fn drop(&mut self) {
        self.app_handle.unlisten(self.unlisten_id);
    }
}

pub struct AwaitWindowCloseEvent<R: Runtime> {
    pub state: Arc<Mutex<StateSimple>>,
    window: WebviewWindow<R>,

}

pub struct StateSimple {
    completed: bool,
    payload: Option<DialogResult>,
    waker: Option<Waker>,
}

impl<R: Runtime> AwaitWindowCloseEvent<R> {
    pub fn new(win: &WebviewWindow<R>) -> Self {
        let state = Arc::new(Mutex::new(StateSimple {
            completed: false,
            payload: None,
            waker: None,
        }));
        let state_clone = state.clone();
        win.on_window_event(move |event| {
            let mut app_state = state_clone.lock().unwrap();
            if let  WindowEvent::CloseRequested { api, .. } = event {
                let result = DialogResult::Closed;
                app_state.completed = true;
                app_state.payload = Some(result);
                if let Some(waker) = app_state.waker.take() {
                    waker.wake();
                }
            }
            
        });
        AwaitWindowCloseEvent::<R> { state, window: win.clone() }
    }
}

impl<R: Runtime> std::future::Future for AwaitWindowCloseEvent<R> {
    type Output = Option<DialogResult>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.completed {
            Poll::Ready(state.payload.take())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

// ^^^ We can also match an enum with

#[pin_project]
pub struct DialogResultFuture<AwaitEvent, AwaitWindowCloseEvent> {
    #[pin]
    pub result_event: AwaitEvent,
    #[pin]
    pub close_event: AwaitWindowCloseEvent,
}

impl<AwaitEvent, AwaitWindowCloseEvent> Future 
    for DialogResultFuture<AwaitEvent, AwaitWindowCloseEvent> 
where 
    AwaitEvent: Future<Output = Option<DialogResult>>,
    AwaitWindowCloseEvent: Future<Output = Option<DialogResult>>,
{
    type Output = Option<DialogResult>;
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let this = self.project();
        
        // let result_event = this.result_event;
        if let Poll::Ready(result) = this.result_event.poll(cx) {
            return Poll::Ready(result);
        }
        
        // let close_event = this.close_event;
        if let Poll::Ready(result) = this.close_event.poll(cx) {
            return Poll::Ready(result);
        }

        Poll::Pending
    }
}
