use closure::closure;

use yew::html::ChangeData::*;
use yew::prelude::*;
use yew_functional::function_component;
use yew_functional::use_state;
use yew_services::{reader::FileData, ReaderService};

use crate::model::{
    serialize::{Serialize::*, *},
    Action,
    Action::*,
};

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub dispatch: Callback<Action>,
}

#[function_component(ProjectView)]
pub fn project_view(props: &Props) -> Html {
    let export = props.dispatch.reform(|_| Serialize(Export));
    let dispatch = &props.dispatch;
    let import: Callback<ChangeData> = Callback::from(closure!(clone dispatch, |evt| {
        let (_, set_reader_task) = use_state(|| None);
        if let Files(filelist) = evt {
            let file = filelist.get(0).unwrap();
            let callback = Callback::from(
                closure!(clone dispatch, clone set_reader_task, |fd: FileData| {
                    let data: Data = fd.content.into();
                    let (signature, workspace) = data.into();
                    dispatch.emit(Serialize(Import(signature, workspace)));
                    set_reader_task(None)
                }),
            );
            let task = ReaderService::read_file(file, callback).expect("failed to read file");
            set_reader_task(Some(task))
        }
    }));
    html! {
        <aside class="project drawer">
            <div class="drawer__header">
                <span class="drawer__title">
                    {"Project"}
                </span>
            </div>
            <div class="drawer__content">
                <button onclick=export>{"Export"}</button>
                <label for="import">
                    {"Import"}
                </label>
                <input type="file" accept="application/msgpack,.hom" class="visually-hidden" id="import" onchange=import/>
            </div>
        </aside>
    }
}