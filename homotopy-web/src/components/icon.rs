use yew::prelude::*;
use yew_functional::function_component;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub name: String,
    pub size: IconSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IconSize {
    Icon18,
    Icon24,
    // Icon36,
    // Icon48,
}

#[function_component(Icon)]
pub fn icon(props: &Props) -> Html {
    let class = format!(
        "material-icons md-light {}",
        match props.size {
            IconSize::Icon18 => "md-18",
            IconSize::Icon24 => "md-24",
            // IconSize::Icon36 => "md-36",
            // IconSize::Icon48 => "md-48",
        }
    );

    html! {
        <i class={class}>{&props.name}</i>
    }
}
