// Integration tests using API v1.
use openfoodfacts::{self as off, Locale, Output};
use reqwest::StatusCode;

#[test]
fn taxonomy() {
    let client = off::v0().build().unwrap();
    let response = client.taxonomy("nova_groups").unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/data/taxonomies/nova_groups.json"
    );
    assert!(response.status().is_success());
}

#[test]
fn taxonomy_not_found() {
    let client = off::v0().build().unwrap();
    let response = client.taxonomy("not_found").unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/data/taxonomies/not_found.json"
    );
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn facet() {
    let client = off::v0().build().unwrap();
    let response = client.facet("brands", None).unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/brands.json"
    );
    assert!(response.status().is_success());
}

#[test]
fn facet_params() {
    let client = off::v0().build().unwrap();
    let output = Output::new()
        .locale(Locale::new("fr", None))
        .page(22)
        .fields("url")
        .nocache(true);
    let response = client.facet("brands", Some(output)).unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://fr.openfoodfacts.org/brands.json?page=22&fields=url&nocache=true"
    );
    assert!(response.status().is_success());
}

#[test]
fn categories() {
    let client = off::v0().build().unwrap();
    let response = client.categories(None).unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/categories.json"
    );
    assert!(response.status().is_success());
}

#[test]
fn categories_params() {
    let client = off::v0().build().unwrap();
    // Accepts only the locale parameter.
    let output = Output::new().locale(Locale::new("fr", None)).page(22);
    let response = client.categories(Some(output)).unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://fr.openfoodfacts.org/categories.json"
    );
    assert!(response.status().is_success());
}

#[test]
fn nutrients() {
    let client = off::v0().build().unwrap();
    let response = client.nutrients(None).unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/cgi/nutrients.pl"
    );
    assert!(response.status().is_success());
}

#[test]
fn nutrients_params() {
    let client = off::v0().build().unwrap();
    // Accepts only the locale parameter.
    let output = Output::new().locale(Locale::new("fr", None)).page(22);
    let response = client.nutrients(Some(output)).unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://fr.openfoodfacts.org/cgi/nutrients.pl"
    );
    assert!(response.status().is_success());
}

#[test]
fn products_by_facet() {
    let client = off::v0().build().unwrap();
    let response = client
        .products_by("additive", "e322-lecithins", None)
        .unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/additive/e322-lecithins.json"
    );
    assert!(response.status().is_success());
}

#[test]
fn products_by_facet_params() {
    let client = off::v0().build().unwrap();
    let output = Output::new()
        .locale(Locale::new("fr", None))
        .pagination(22, 20)
        .fields("url");
    let response = client
        .products_by("additif", "e322-lecithines", Some(output))
        .unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://fr.openfoodfacts.org/additif/e322-lecithines.json?page=22&page_size=20&fields=url"
    );
    assert!(response.status().is_success());
}

#[test]
fn products_by_category() {
    let client = off::v0().build().unwrap();
    let response = client.products_by("category", "cheeses", None).unwrap();
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/category/cheeses.json"
    );
    assert!(response.status().is_success());
}

#[test]
fn products_by_category_params() {
    let client = off::v0().build().unwrap();
    let output = Output::new()
        .locale(Locale::new("fr", None))
        .pagination(22, 20)
        .fields("url");
    let response = client
        .products_by("categorie", "fromages", Some(output))
        .unwrap();

    assert_eq!(
        response.url().as_str(),
        "https://fr.openfoodfacts.org/categorie/fromages.json?page=22&page_size=20&fields=url"
    );
    assert!(response.status().is_success());
}

#[test]
fn product() {
    let client = off::v0().build().unwrap();
    let response = client.product("069000019832", None).unwrap(); // Diet Pepsi
    assert_eq!(
        response.url().as_str(),
        "https://world.openfoodfacts.org/api/v0/product/069000019832"
    );
    assert!(response.status().is_success());
}

#[test]
fn product_params() {
    let client = off::v0().build().unwrap();
    // Accepts only the locale and fields parameters.
    let output = Output::new()
        .locale(Locale::new("fr", None))
        .pagination(22, 20)
        .fields("url");
    let response = client.product("069000019832", Some(output)).unwrap(); // 069000019832 = Diet Pepsi
    assert_eq!(
        response.url().as_str(),
        "https://fr.openfoodfacts.org/api/v0/product/069000019832?fields=url"
    );
    assert!(response.status().is_success());
}

#[test]
fn search_v0() {
    let client = off::v0().build().unwrap();
    let query = client
        .query()
        .criteria("brands", "contains", "Nestlé")
        .criteria("categories", "does_not_contain", "cheese")
        .ingredient("additives", "without")
        .ingredient("ingredients_that_may_be_from_palm_oil", "indifferent")
        .nutrient("fiber", "lt", 500)
        .nutrient("salt", "gt", 100)
        .terms("cereals");

    let response = client.search(query, None).unwrap();
    assert_eq!(response.url().path(), "/cgi/search.pl");
    assert!(response.status().is_success());
}

#[test]
fn search_v2() {
    let client = off::v2().build().unwrap();
    let query = client
        .query()
        .criteria("brands", "Nestlé", Some("fr"))
        .criteria("categories", "-cheese", None)
        .nutrient_100g("fiber", "<", 500)
        .nutrient_serving("salt", "=", 100);

    let response = client.search(query, None).unwrap();
    assert_eq!(response.url().path(), "/api/v2/search");
    assert!(response.status().is_success());
}
