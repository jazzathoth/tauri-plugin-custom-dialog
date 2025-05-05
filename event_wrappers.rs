use std::{
    pin::Pin, 
    task::{Context, Poll, Waker},
    sync::{Arc, Mutex},
    future::Future,
};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use tauri::{AppHandle, Event, Listener, WebviewWindow, WindowEvent};
use pin_project::pin_project;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="status")]
pub enum DialogResult {
  Confirm { data: Option<JsonValue> },
  Cancel,
  Closed,
}

pub struct AwaitEvent {
    pub state: Arc<Mutex<State>>,
}

pub struct State {
    completed: bool,
    payload: Option<DialogResult>,
    waker: Option<Waker>,
    unlisten_id: Option<u32>,
    app_handle: tauri::AppHandle,
}

impl AwaitEvent {
    pub fn new(app: AppHandle, event_name: &str) -> Self {
        let state = Arc::new(Mutex::new(State {
            completed: false,
            payload: None,
            waker: None,
            unlisten_id: None,
            app_handle: app.clone(),
        }));
        let state_clone = state.clone();
        let state_clone_inner = state.clone();

        let listener_id = app.listen(event_name, move |event: Event| {
            let mut state_unwrapped_inner = state_clone_inner.lock().unwrap();
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

        let mut state_unwrapped = state_clone.lock().unwrap();
        state_unwrapped.unlisten_id = Some(listener_id);
        AwaitEvent { state }
    }
    // pub fn cleanup(&self) {
    //     let mut state = self.state.lock().unwrap();
    //     if let Some(event_id) = state.unlisten_id.take() {
    //         state.app_handle.unlisten(event_id);
    //     }
    // }
}

impl std::future::Future for AwaitEvent {
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

impl Drop for AwaitEvent {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.try_lock() {
            if let Some(event_id) = state.unlisten_id.take() {
                state.app_handle.unlisten(event_id);
            }
        } else { eprintln!("Error cleaning up future because could not aquire lock")}
    }
}

pub struct AwaitWindowCloseEvent {
    pub state: Arc<Mutex<StateSimple>>,
}

pub struct StateSimple {
    completed: bool,
    payload: Option<DialogResult>,
    waker: Option<Waker>,
}

impl AwaitWindowCloseEvent {
    pub fn new(win: &WebviewWindow) -> Self {
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
        AwaitWindowCloseEvent { state }
    }
}

impl std::future::Future for AwaitWindowCloseEvent {
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
