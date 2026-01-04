//! Resource bar component for displaying game resources.

use leptos::prelude::*;

/// A single resource to display.
#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    /// Display name/label
    pub name: String,
    /// Icon (emoji or symbol)
    pub icon: String,
    /// Current value
    pub value: i64,
    /// Maximum value (for percentage calculations)
    pub max: Option<i64>,
    /// Threshold below which to show "low" warning
    pub low_threshold: Option<i64>,
    /// Threshold below which to show "critical" warning
    pub critical_threshold: Option<i64>,
}

impl Resource {
    pub fn new(name: impl Into<String>, icon: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            icon: icon.into(),
            value,
            max: None,
            low_threshold: None,
            critical_threshold: None,
        }
    }

    pub fn with_max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn with_thresholds(mut self, low: i64, critical: i64) -> Self {
        self.low_threshold = Some(low);
        self.critical_threshold = Some(critical);
        self
    }

    fn value_class(&self) -> &'static str {
        if let Some(critical) = self.critical_threshold {
            if self.value <= critical {
                return "critical";
            }
        }
        if let Some(low) = self.low_threshold {
            if self.value <= low {
                return "low";
            }
        }
        ""
    }
}

/// Display a single resource.
#[component]
pub fn ResourceDisplay(
    /// The resource to display
    resource: Resource,
) -> impl IntoView {
    let value_class = resource.value_class();
    let display_value = if let Some(max) = resource.max {
        format!("{}/{}", resource.value, max)
    } else {
        resource.value.to_string()
    };

    view! {
        <div class="bw-resource" title=resource.name.clone()>
            <span class="bw-resource-icon">{resource.icon}</span>
            <span class=format!("bw-resource-value {}", value_class)>
                {display_value}
            </span>
        </div>
    }
}

/// Display multiple resources in a bar.
#[component]
pub fn ResourceBar(
    /// The resources to display
    resources: RwSignal<Vec<Resource>>,
) -> impl IntoView {
    view! {
        <div class="bw-resource-bar">
            {move || {
                resources.get().into_iter().map(|resource| {
                    view! { <ResourceDisplay resource=resource /> }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}

/// Static resource bar (non-reactive).
#[component]
pub fn StaticResourceBar(
    /// The resources to display
    resources: Vec<Resource>,
) -> impl IntoView {
    view! {
        <div class="bw-resource-bar">
            {resources.into_iter().map(|resource| {
                view! { <ResourceDisplay resource=resource /> }
            }).collect::<Vec<_>>()}
        </div>
    }
}
