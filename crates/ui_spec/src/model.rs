pub const UI_SPEC_VERSION: &str = "2.0";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UiSpec {
    Container {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "String::is_empty")]
        text: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Instance {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Text {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Image {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Shape {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Vector {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
}

impl Default for UiSpec {
    fn default() -> Self {
        Self::Container {
            id: String::new(),
            name: String::new(),
            text: String::new(),
            children: Vec::new(),
        }
    }
}

impl UiSpec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_pretty_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    pub fn to_pretty_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new().struct_names(true))
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Container { id, .. }
            | Self::Instance { id, .. }
            | Self::Text { id, .. }
            | Self::Image { id, .. }
            | Self::Shape { id, .. }
            | Self::Vector { id, .. } => id.as_str(),
        }
    }

    pub fn children(&self) -> &[UiSpec] {
        match self {
            Self::Container { children, .. }
            | Self::Instance { children, .. }
            | Self::Text { children, .. }
            | Self::Image { children, .. }
            | Self::Shape { children, .. }
            | Self::Vector { children, .. } => children.as_slice(),
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Container { .. } => NodeType::Container,
            Self::Instance { .. } => NodeType::Instance,
            Self::Text { .. } => NodeType::Text,
            Self::Image { .. } => NodeType::Image,
            Self::Shape { .. } => NodeType::Shape,
            Self::Vector { .. } => NodeType::Vector,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Container,
    Instance,
    Text,
    Image,
    Shape,
    Vector,
}
