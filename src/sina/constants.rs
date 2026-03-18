pub const SINA_DATA_ASSETS: &[(&str, &str)] = &[
    (
        "one_gram",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/one_gram.json?download=true",
    ),
    (
        "five_grams",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/five_grams.json?download=true",
    ),
    (
        "four_grams",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/four_grams.json?download=true",
    ),
    (
        "three_grams",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/three_grams.json?download=true",
    ),
    (
        "two_grams",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/two_grams.json?download=true",
    ),
    (
        "graph_l2",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/graph_l2.json?download=true",
    ),
    (
        "graph_l3",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/graph_l3.json?download=true",
    ),
    (
        "morph",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/morph.json?download=true",
    ),
    (
        "catalogue",
        "https://huggingface.co/datasets/hadikhamoud/fast-disambig-data/resolve/main/sina/catalogue.json?download=true",
    ),
];

pub fn sina_asset_url(key: &str) -> Option<&'static str> {
    SINA_DATA_ASSETS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}
