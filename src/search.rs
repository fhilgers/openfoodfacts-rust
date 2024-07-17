use crate::client::{RequestMethods, Result, SearchUrl};
use crate::output::Output;
use crate::types::Params;
use std::fmt::{self, Display, Formatter};

/// Sorting criteria.
///
/// # Variants:
///
/// * Popularity - Number of unique scans.
/// * Product name - Product name, alphabetical.
/// * CreatedDate - Add date.
/// * LastModifiedDate - Last edit date.
/// * EcoScore - Eco score.
///
/// TODO:
/// last_modified_t_complete_first
/// scans_n
/// completeness
/// popularity_key
/// popularity
/// nutriscore_score
/// nova_score
/// nothing
#[derive(Debug)]
pub enum SortBy {
    Popularity,
    ProductName,
    CreatedDate,
    LastModifiedDate,
    EcoScore,
}

impl Display for SortBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let sort = match self {
            Self::Popularity => "unique_scans_n",
            Self::ProductName => "product_name",
            Self::CreatedDate => "created_t",
            Self::LastModifiedDate => "last_modified_t",
            Self::EcoScore => "ecoscore_score",
        };
        write!(f, "{}", sort)
    }
}

/// Builds a search query.
///
/// Concrete types must implement the [crate::search::QueryParams] trait.
#[derive(Debug, Default)]
pub struct SearchQuery<S> {
    params: Vec<(String, Value)>,
    sort_by: Option<SortBy>,
    state: S,
}

// The internal representation of a search query parameter value.
#[derive(Debug)]
enum Value {
    String(String),
    Number(u32),
    None,
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Number(value)
    }
}

/// Converts a SearchQuery<S> object into a [crate::types::Params] object.
pub trait QueryParams {
    fn params(&self) -> Params;
}

impl<S> SearchQuery<S> {
    /// Sets the sorting order.
    pub fn sort_by(mut self, sort_by: SortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    /// Sends the search query. Relies on the client to obtain the versioned
    /// search API endpoint and to send the request.
    pub(crate) fn search(
        params: impl QueryParams,
        client: &(impl SearchUrl + RequestMethods),
        output: Option<Output>,
    ) -> Result {
        let url = client.search_url(output.as_ref().and_then(|o| o.locale.as_ref()))?;
        let mut params = params.params();
        if let Some(output_params) = output.map(|o| o.params(&["page", "page_size", "fields"])) {
            params.extend(output_params);
        }
        client.get(url, Some(&params))
    }
}

// ----------------------------------------------------------------------------
// SearchQuery V0
// ----------------------------------------------------------------------------

/// A search query builder for the Search API V0.
///
/// # Examples
///
/// ```
/// use openfoodfacts as off;
///
/// # fn main() -> Result<(), off::Error> {
/// let client = off::v0().build().unwrap();
/// let query = client
///     .query()
///     .criteria("categories", "contains", "cereals")
///     .criteria("label", "contains", "kosher")
///     .ingredient("additives", "without")
///     .nutrient("energy", "lt", 500);
/// let response = client.search(query, None)?;
/// assert!(response.status().is_success());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct QueryStateV0 {
    criteria_index: u32,
    nutrient_index: u32,
}

pub type SearchQueryV0 = SearchQuery<QueryStateV0>;

impl SearchQueryV0 {
    /// Defines a criteria query parameter producing a triplet of pairs
    ///
    /// ```code
    /// tagtype_N=<criteria>
    /// tag_contains_N=<op>
    /// tag_N=<value>
    /// ```
    ///
    /// # Arguments
    ///
    /// * criteria - A valid criteria name. See the [`API docs`].
    /// * op - One of "contains" or "does_not_contain".
    /// * value - The searched criteria value.
    ///
    /// [`API docs`]: https://openfoodfacts.github.io/api-documentation/#5Filtering
    pub fn criteria(mut self, criteria: &str, op: &str, value: &str) -> Self {
        self.state.criteria_index += 1;
        self.params.push((
            format!("tagtype_{}", self.state.criteria_index),
            Value::from(criteria),
        ));
        self.params.push((
            format!("tag_contains_{}", self.state.criteria_index),
            Value::from(op),
        ));
        self.params.push((
            format!("tag_{}", self.state.criteria_index),
            Value::from(value),
        ));
        self
    }

    /// Defines an ingredient query parameter, producing a pair
    ///
    /// `<ingredient>=<value>`
    ///
    /// # Arguments
    ///
    /// * ingredient - One of:
    ///     - "additives"
    ///     - "ingredients_from_palm_oil",
    ///     - "ingredients_that_may_be_from_palm_oil",
    ///     - "ingredients_from_or_that_may_be_from_palm_oil".
    // * value: One of "with", "without", "indifferent".
    ///
    /// If `ingredient` is "additives", the values "with", "without" and "indiferent"
    /// are converted to "with_additives", "without_additives" and "indifferent_additives"
    /// respectively.
    pub fn ingredient(mut self, ingredient: &str, value: &str) -> Self {
        self.params.push((
            String::from(ingredient),
            match ingredient {
                "additives" => Value::from(format!("{}_additives", value)),
                _ => Value::from(value),
            },
        ));
        self
    }

    /// Defines a nutrient (a.k.a nutriment in the API docs) search parameter,
    /// producing a triplet of pairs
    ///
    /// ```code
    /// nutriment_N=<nutriment>
    /// nutriment_compare_N=<op>
    /// nutriment_value_N=<quantity>
    /// ```
    ///
    /// # Arguments
    ///
    /// * nutrient - The nutrient name. See the [`API docs`].
    /// * op - The comparation operation to perform. One of "lt", "lte", "gt", "gte",
    ///        "eq".
    /// * value - The value to compare.
    ///
    /// [`API docs`]: https://openfoodfacts.github.io/api-documentation/#5Filtering
    pub fn nutrient(mut self, nutriment: &str, op: &str, value: u32) -> Self {
        self.state.nutrient_index += 1;
        self.params.push((
            format!("nutriment_{}", self.state.nutrient_index),
            Value::from(nutriment),
        ));
        self.params.push((
            format!("nutriment_compare_{}", self.state.nutrient_index),
            Value::from(op),
        ));
        self.params.push((
            format!("nutriment_value_{}", self.state.nutrient_index),
            Value::from(value),
        ));
        self
    }

    pub fn terms(mut self, search_terms: &str) -> Self {
        self.params.push((
            format!("search_terms"),
            Value::from(search_terms),
        ));
        self
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl QueryParams for SearchQueryV0 {
    fn params(&self) -> Params {
        let mut params: Params = Vec::new();
        for (name, value) in &self.params {
            let v = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::None => {
                    continue;
                }
            };
            params.push((name, v));
        }
        if let Some(ref s) = self.sort_by {
            params.push(("sort_by", s.to_string()));
        }
        // Adds the 'action' and 'json' parameter. TODO: Should be done in client::search() ?
        params.push(("action", String::from("process")));
        params.push(("json", true.to_string()));
        params
    }
}

// ----------------------------------------------------------------------------
// Search Query V2
// ----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct QueryStateV2;

pub type SearchQueryV2 = SearchQuery<QueryStateV2>;

impl SearchQueryV2 {
    /// Defines a criteria query parameter, producing pairs
    ///
    /// `<criteria>_tags=<value>`
    ///
    /// or
    ///
    /// `<criteria>_tags_<lc>= <value>`
    ///
    /// if a language code has been given.
    ///
    /// # Arguments
    ///
    /// * criteria - A valid criteria name. See the [`API docs`].
    /// * value - The criteria value. Use comma for AND, colon for OR and tilde for NOT.
    ///     See the [`Search V2 API docs`].
    /// * lc: Optional language code.
    ///
    /// [`openfoodfacts API docs`]: https://openfoodfacts.github.io/api-documentation/#5Filtering
    /// [`Search V2 API docs`]: https://wiki.openfoodfacts.org/Open_Food_Facts_Search_API_Version_2
    pub fn criteria(mut self, criteria: &str, value: &str, lc: Option<&str>) -> Self {
        if let Some(lc) = lc {
            self.params
                .push((format!("{}_tags_{}", criteria, lc), Value::from(value)));
        } else {
            self.params
                .push((format!("{}_tags", criteria), Value::from(value)));
        }
        self
    }

    /// Defines a condition on a nutrient, producing a pair
    ///
    /// `<nutrient>_<unit>=<value>`
    ///
    /// if `op` is "=", otherwise produces a non-valued parameter:
    ///
    /// `<nutient>_<unit><op><value>`
    ///
    /// # Arguments
    ///
    /// * nutrient - The nutrient name. See the [`API docs`].
    /// * unit - One of the "100g" or "serving".
    /// * op - A comparison operator. One of  '=', '<', '>', `<=', '=>`.
    ///     See the [`Search V2 API docs`].
    /// * value - The value to compare.
    ///
    /// TODO: Verify the <= and => operators.
    ///
    /// [`API docs`]: https://openfoodfacts.github.io/api-documentation/#5Filtering
    /// [`Search V2 API docs`]: https://wiki.openfoodfacts.org/Open_Food_Facts_Search_API_Version_2
    pub fn nutrient(mut self, nutrient: &str, unit: &str, op: &str, value: u32) -> Self {
        let param = match op {
            "=" => (format!("{}_{}", nutrient, unit), Value::from(value)),
            // The name and value becomes the param name. TODO: Check HTTP specs if <, >, etc supported
            // in query params in place of =.
            _ => (format!("{}_{}{}{}", nutrient, unit, op, value), Value::None),
        };
        self.params.push(param);
        self
    }

    /// Convenience method to add a nutrient condition per 100 grams.
    pub fn nutrient_100g(self, nutrient: &str, op: &str, value: u32) -> Self {
        self.nutrient(nutrient, "100g", op, value)
    }

    /// Convenience method to add a nutrient condition per serving.
    pub fn nutrient_serving(self, nutrient: &str, op: &str, value: u32) -> Self {
        self.nutrient(nutrient, "serving", op, value)
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl QueryParams for SearchQueryV2 {
    fn params(&self) -> Params {
        let mut params: Params = Vec::new();
        for (name, value) in &self.params {
            let v = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::None => String::new(), // The empty string
            };
            params.push((name, v));
        }
        if let Some(ref s) = self.sort_by {
            params.push(("sort_by", s.to_string()));
        }
        params
    }
}

#[cfg(test)]
mod tests_sort_by {
    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(
            SortBy::Popularity.to_string(),
            String::from("unique_scans_n")
        );
        assert_eq!(
            SortBy::ProductName.to_string(),
            String::from("product_name")
        );
        assert_eq!(SortBy::CreatedDate.to_string(), String::from("created_t"));
        assert_eq!(
            SortBy::LastModifiedDate.to_string(),
            String::from("last_modified_t")
        );
    }
}

#[cfg(test)]
mod tests_search_v0 {
    use super::*;

    #[test]
    fn query_params() {
        let query = SearchQueryV0::new()
            .criteria("brands", "contains", "Nestlé")
            .criteria("categories", "does_not_contain", "cheese")
            .ingredient("additives", "without")
            .ingredient("ingredients_that_may_be_from_palm_oil", "indifferent")
            .nutrient("fiber", "lt", 500)
            .nutrient("salt", "gt", 100)
            .terms("cereal");

        let params = query.params();
        assert_eq!(
            &params,
            &[
                ("tagtype_1", String::from("brands")),
                ("tag_contains_1", String::from("contains")),
                ("tag_1", String::from("Nestlé")),
                ("tagtype_2", String::from("categories")),
                ("tag_contains_2", String::from("does_not_contain")),
                ("tag_2", String::from("cheese")),
                ("additives", String::from("without_additives")),
                (
                    "ingredients_that_may_be_from_palm_oil",
                    String::from("indifferent")
                ),
                ("nutriment_1", String::from("fiber")),
                ("nutriment_compare_1", String::from("lt")),
                ("nutriment_value_1", String::from("500")),
                ("nutriment_2", String::from("salt")),
                ("nutriment_compare_2", String::from("gt")),
                ("nutriment_value_2", String::from("100")),
                ("search_terms", String::from("cereal")),
                ("action", String::from("process")),
                ("json", String::from("true"))
            ]
        );
    }
}

#[cfg(test)]
mod tests_search_v2 {
    use super::*;

    #[test]
    fn search_params() {
        let query = SearchQueryV2::new()
            .criteria("brands", "Nestlé", Some("fr"))
            .criteria("categories", "-cheese", None)
            // TODO ?
            //              .ingredient("additives", "without")
            //              .ingredient("ingredients_that_may_be_from_palm_oil", "indifferent")
            .nutrient_100g("fiber", "<", 500)
            .nutrient_serving("salt", "=", 100);

        let params = query.params();
        assert_eq!(
            &params,
            &[
                ("brands_tags_fr", String::from("Nestlé")),
                ("categories_tags", String::from("-cheese")),
                // TODO
                //            ("additives", String::from("without_additives")),
                //            ("ingredients_that_may_be_from_palm_oil", String::from("indifferent")),
                ("fiber_100g<500", String::new()),
                ("salt_serving", String::from("100")),
            ]
        );
    }
}
