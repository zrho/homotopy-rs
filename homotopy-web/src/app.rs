use yew::agent::Dispatcher;
use yew::prelude::*;

use wasm_bindgen::closure::Closure;

use crate::components::icon::{Icon, IconSize};
use crate::components::toast::{Toast, ToastAgent, Toaster};
use crate::model;

mod attach;
mod diagram2d;
mod panzoom;
mod project;
mod settings;
mod sidebar;
mod signature;
mod signature_stylesheet;
mod util;
mod workspace;

use settings::AppSettings;
use sidebar::Sidebar;
use signature_stylesheet::SignatureStylesheet;
use workspace::WorkspaceView;

#[derive(Default, Clone, Debug, PartialEq, Properties)]
pub struct Props {}

#[derive(Debug, Clone)]
pub enum Message {
    Dispatch(model::Action),
}

pub struct App {
    dispatch: Callback<model::Action>,
    state: model::State,
    signature_stylesheet: SignatureStylesheet,
    toaster: Dispatcher<ToastAgent>,
    _settings: AppSettings,
    before_unload: Option<Closure<dyn FnMut(web_sys::BeforeUnloadEvent)>>,
}

impl Component for App {
    type Message = Message;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = model::State::default();
        let dispatch = link.callback(Message::Dispatch);

        // Install the signature stylesheet
        let mut signature_stylesheet = SignatureStylesheet::new("generator");
        signature_stylesheet.update(state.with_proof(|p| p.signature().clone()));
        signature_stylesheet.mount();

        let mut app = Self {
            dispatch,
            state,
            signature_stylesheet,
            toaster: ToastAgent::dispatcher(),
            _settings: AppSettings::connect(Callback::noop()),
            before_unload: None,
        };
        app.install_unload_hook();
        app
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Dispatch(action) => {
                log::info!("Received action: {:?}", action);

                let time_start = performance();
                let result = self.state.update(action);
                let time_stop = performance();
                log::info!("State update took {}ms.", time_stop - time_start);

                homotopy_core::collect_garbage();

                match result {
                    Ok(()) => {}
                    Err(error) => {
                        self.toaster.send(Toast::error(format!("{}", error)));
                        log::error!("Error occured: {}", error);
                    }
                }
                self.signature_stylesheet
                    .update(self.state.with_proof(|p| p.signature().clone()));
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let dispatch = &self.dispatch;
        let proof = self.state.with_proof(Clone::clone);
        let signature = proof.signature();

        let workspace = match proof.workspace() {
            Some(workspace) => {
                html! {
                    <WorkspaceView
                        workspace={workspace}
                        signature={signature}
                        dispatch={dispatch.reform(model::Action::Proof)}
                    />
                }
            }
            None => {
                // TODO: Show onboarding info if workspace and signature is empty
                html! {
                    <content class="workspace workspace--empty">
                    </content>
                }
            }
        };

        html! {
            <main class="app">
                <Sidebar dispatch={dispatch} proof={self.state.with_proof(Clone::clone)}/>
                <Toaster timeout={3000} />
                {workspace}
                <span class="version">
                    {format!("Version: {}", option_env!("GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION")))}
                </span>
            </main>
        }
    }

    fn destroy(&mut self) {
        self.signature_stylesheet.unmount();
    }
}

impl App {
    fn install_unload_hook(&mut self) {
        use wasm_bindgen::JsCast;

        let before_unload = Closure::wrap(Box::new(move |event: web_sys::BeforeUnloadEvent| {
            event.set_return_value("Are you sure you want to leave? Unsaved changes will be lost!");
        }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .set_onbeforeunload(Some(before_unload.as_ref().unchecked_ref()));

        self.before_unload = Some(before_unload);
    }
}

fn performance() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}
