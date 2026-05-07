#[derive(Default, Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct List {
    pub items: Vec<usize>,
}
