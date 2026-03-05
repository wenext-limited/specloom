pub const UI_SPEC_VERSION: &str = "2.0";

macro_rules! define_ui_spec_enum {
    (
        containers: [$($container_variant:ident),+ $(,)?],
        leaves: [$($leaf_variant:ident),+ $(,)?],
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum UiSpec {
            $(
                $container_variant {
                    id: String,
                    name: String,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "String::is_empty")]
                    text: String,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    children: Vec<UiSpec>,
                },
            )+
            $(
                $leaf_variant {
                    id: String,
                    name: String,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    children: Vec<UiSpec>,
                },
            )+
        }
    };
}

define_ui_spec_enum! {
    containers: [Container, Button],
    leaves: [Instance, Text, Image, Shape, Vector, ScrollView, HStack, VStack, ZStack],
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
            | Self::Vector { id, .. }
            | Self::Button { id, .. }
            | Self::ScrollView { id, .. }
            | Self::HStack { id, .. }
            | Self::VStack { id, .. }
            | Self::ZStack { id, .. } => id.as_str(),
        }
    }

    pub fn children(&self) -> &[UiSpec] {
        match self {
            Self::Container { children, .. }
            | Self::Instance { children, .. }
            | Self::Text { children, .. }
            | Self::Image { children, .. }
            | Self::Shape { children, .. }
            | Self::Vector { children, .. }
            | Self::Button { children, .. }
            | Self::ScrollView { children, .. }
            | Self::HStack { children, .. }
            | Self::VStack { children, .. }
            | Self::ZStack { children, .. } => children.as_slice(),
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
            Self::Button { .. } => NodeType::Button,
            Self::ScrollView { .. } => NodeType::ScrollView,
            Self::HStack { .. } => NodeType::HStack,
            Self::VStack { .. } => NodeType::VStack,
            Self::ZStack { .. } => NodeType::ZStack,
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
    Button,
    ScrollView,
    HStack,
    VStack,
    ZStack,
}
