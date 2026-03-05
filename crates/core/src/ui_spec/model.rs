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
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    repeat_element_ids: Vec<String>,
                },
            )+
            $(
                $leaf_variant {
                    id: String,
                    name: String,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    children: Vec<UiSpec>,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    repeat_element_ids: Vec<String>,
                },
            )+
        }

        impl UiSpec {
            pub fn node_type(&self) -> NodeType {
                match self {
                    $(
                        UiSpec::$container_variant { .. } => NodeType::$container_variant,
                    )+
                    $(
                        UiSpec::$leaf_variant { .. } => NodeType::$leaf_variant,
                    )+
                }
            }

            pub fn id(&self) -> &str {
                match self {
                    $(
                        UiSpec::$container_variant { id, .. } => id.as_str(),
                    )+
                    $(
                        UiSpec::$leaf_variant { id, .. } => id.as_str(),
                    )+
                }
            }

            pub fn children(&self) -> &[UiSpec] {
                match self {
                    $(
                        UiSpec::$container_variant { children, .. } => children.as_slice(),
                    )+
                    $(
                        UiSpec::$leaf_variant { children, .. } => children.as_slice(),
                    )+
                }
            }

            pub fn repeat_element_ids(&self) -> &[String] {
                match self {
                    $(
                        UiSpec::$container_variant { repeat_element_ids, .. } => repeat_element_ids.as_slice(),
                    )+
                    $(
                        UiSpec::$leaf_variant { repeat_element_ids, .. } => repeat_element_ids.as_slice(),
                    )+
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum NodeType {
            $(
                $container_variant,
            )+
            $(
                $leaf_variant,
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
            repeat_element_ids: Vec::new(),
        }
    }
}

impl UiSpec {
    pub fn to_pretty_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new().struct_names(true))
    }
}
