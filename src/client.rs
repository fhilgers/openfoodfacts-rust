// Notes:
//
// * The 'cc' and 'lc' query parmeters are not supported. The country and
//   language are always selected via the subdomain.
// * Only JSON calls are supported.
use crate::locale::Locale;
use crate::output::Output;
use crate::search::{SearchQueryV0, SearchQueryV2};
use crate::types::{Params, Version, V0, V2};
pub use reqwest::blocking::{Client as HttpClient, Response as HttpResponse};
use url::{ParseError, Url};

/// The error type of all OffClient methods.
pub type Error = Box<dyn std::error::Error>;

/// The return type of all OffClient methods.
pub type Result = std::result::Result<HttpResponse, Error>;

/// The OFF API client.
///
/// The client owns a [reqwest::Client] object. One single OFF client should
/// be used per application.
///
/// All methods return an [OffResult] object.
#[derive(Debug)]
pub struct OffClient<V> {
    // The version marker.
    v: V,
    // The default locale to use when no locale is given in a method call.
    locale: Locale,
    // The uderlying reqwest client.
    client: HttpClient,
}

/// Generates common OFF Urls.
///
/// This trait provides the default implementations. Concrete types need only to
/// implement the host_with_locale() method.
pub(crate) trait Urls {
    /// Return the base URL with the given locale or the default locale if
    /// none given.
    fn base_url(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError> {
        self.host_with_locale(locale)
    }

    /// Return the base URL with the "world" locale.
    fn base_url_world(&self) -> std::result::Result<Url, ParseError> {
        self.host_with_locale(Some(&Locale::default()))
    }

    /// Return the CGI URL with the locale given locale or the default locale if
    /// none given.
    fn cgi_url(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError> {
        let base = self.base_url(locale)?;
        base.join("cgi/")
    }

    // Return the base URL with the given locale. If locale is None, return the
    // client's default locale.
    fn host_with_locale(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError>;
}

/// Generate versioned API URLs.
pub(crate) trait ApiUrl: Version + Urls {
    /// Return the versioned API URL with the given locale or the default locale if
    /// none given.
    fn api_url(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError> {
        let base = self.base_url(locale)?;
        base.join(&format!("api/{}/", self.version()))
    }
}

/// Generate versioned search API URLs.
pub(crate) trait SearchUrl: ApiUrl {
    /// Return the versioned search URL.
    fn search_url(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError>;
}

/// OFF request methods. At present, only GET is implemented.
pub trait RequestMethods {
    /// Build and send a GET request.
    fn get(&self, url: Url, params: Option<&Params>) -> Result;
}

impl<V> Version for OffClient<V>
where
    V: Version,
{
    fn version(&self) -> &str {
        self.v.version()
    }
}

impl<V> Urls for OffClient<V>
where
    V: Version,
{
    /// Returns the base URL with the given locale. If locale is None, return the
    /// client's default locale.
    fn host_with_locale(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError> {
        let url = format!(
            "https://{}.openfoodfacts.org/",
            locale.map_or(self.locale.to_string(), |l| l.to_string())
        );
        Url::parse(&url)
    }
}

impl<V> ApiUrl for OffClient<V> where V: Version {}

impl<V> RequestMethods for OffClient<V> {
    /// Builds and send a GET request.
    fn get(&self, url: Url, params: Option<&Params>) -> Result {
        let mut rb = self.client.get(url);
        if let Some(p) = params {
            rb = rb.query(p);
        }
        let response = rb.send()?;
        Ok(response)
    }
}

impl<V> OffClient<V>
where
    V: Version + Copy,
{
    // ------------------------------------------------------------------------
    // Metadata
    // ------------------------------------------------------------------------

    /// Gets the given taxonomy. Taxonomies are static JSON files.
    ///
    /// # OFF API request
    ///
    /// `GET https://world.openfoodfacts.org/data/taxonomies/{taxonomy}.json`
    ///
    /// Taxonomies support only the locale "world".
    ///
    /// # Arguments
    ///
    /// * taxonomy - The taxonomy name. One of the following:
    ///     - additives
    ///     - allergens
    ///     - additives_classes (*)
    ///     - brands
    ///     - countries
    ///     - ingredients
    ///     - ingredients_analysis (*)
    ///     - languages
    ///     - nova_groups (*)
    ///     - nutrient_levels (*)
    ///     - states
    /// (*) Only taxonomy. There is no facet equivalent.
    pub fn taxonomy(&self, taxonomy: &str) -> Result {
        let base_url = self.base_url_world()?; // force world locale.
        let url = base_url.join(&format!("data/taxonomies/{}.json", taxonomy))?;
        self.get(url, None)
    }

    /// Gets the given facet.
    ///
    /// # OFF API request
    ///
    /// `GET https://{locale}.openfoodfacts.org/{facet}.json`
    ///
    /// # Arguments
    ///
    /// * facet - The facet type name. One of the following:
    ///     - additives
    ///     - allergens
    ///     - brands
    ///     - countries
    ///     - entry-dates
    ///     - ingredients
    ///     - labels
    ///     - languages
    ///     - packaging
    ///     - purchase-places
    ///     - states
    ///     - stores
    ///     - traces
    ///     The name may be given in english or localized, i.e. additives (world), additifs (fr).
    /// * output - Optional output parameters. This call supports only the locale,
    ///     pagination, fields and nocache parameters.
    pub fn facet(&self, facet: &str, output: Option<Output>) -> Result {
        // Borrow output and extract Option<&Locale>
        let base_url = self.base_url(output.as_ref().and_then(|o| o.locale.as_ref()))?;
        let url = base_url.join(&format!("{}.json", facet))?;
        let params = output.map(|o| o.params(&["page", "page_size", "fields", "nocache"]));
        self.get(url, params.as_ref())
    }

    /// Gets all the categories.
    ///
    /// # OFF API request
    ///
    /// `GET https://world.openfoodfacts.org/categories.json`
    ///
    /// # Arguments
    ///
    /// * output - Optional output parameters. This call supports only the locale parameter.
    pub fn categories(&self, output: Option<Output>) -> Result {
        let base_url = self.base_url(output.as_ref().and_then(|o| o.locale.as_ref()))?;
        let url = base_url.join("categories.json")?;
        self.get(url, None)
    }

    /// Gets the nutrients by country.
    ///
    /// # OFF API request
    ///
    /// `GET https://{locale}.openfoodfacts.org/cgi/nutrients.pl`
    ///
    /// # Arguments
    ///
    /// * output - Optional output parameter. This call supports only the locale
    ///   parameter.
    pub fn nutrients(&self, output: Option<Output>) -> Result {
        let cgi_url = self.cgi_url(output.as_ref().and_then(|o| o.locale.as_ref()))?;
        let url = cgi_url.join("nutrients.pl")?;
        self.get(url, None)
    }

    /// Gets all products for the given facet or category.
    ///
    /// # OFF API request
    ///
    /// `GET https://{locale}.openfoodfacts.org/{facet}/{value}.json`
    ///
    /// # Arguments
    ///
    /// * what - A facet name or "category". The facet name is always the singular name
    ///     of the face type name (i.e. brands -> brand, entry-dates -> entry-date, etc).
    ///     The facet name or the "category" literal may be given either in english or
    ///     localized, i.e. additives (world), additifs (fr), category (world), categorie (fr).
    /// * id - The localized id of the facet or category. The IDs are returned by calls
    ///     to the corresponding `facet(<facet_type>)` or `categories()` endpoint. For example,
    ///     the IDs for the `entry-date` facet are returned by the call `facet("entry-dates")`.
    /// * output - Optional output parameters. This call supports the locale, pagination
    ///     and fields parameters.
    pub fn products_by(&self, what: &str, id: &str, output: Option<Output>) -> Result {
        let base_url = self.base_url(output.as_ref().and_then(|o| o.locale.as_ref()))?;
        let url = base_url.join(&format!("{}/{}.json", what, id))?;
        let params = output.map(|o| o.params(&["page", "page_size", "fields"]));
        self.get(url, params.as_ref())
    }

    // ------------------------------------------------------------------------
    // Read
    // ------------------------------------------------------------------------

    /// Gets the nutrition facts of the given product.
    ///
    /// # OFF API request
    ///
    /// `GET https://{locale}.openfoodfacts.org/api/{version}/product/{barcode}`
    ///
    /// # Arguments
    ///
    /// * barcode - The product barcode.
    /// * output - Optional output parameters. This call only supports the locale
    ///     and fields parameters.
    pub fn product(&self, barcode: &str, output: Option<Output>) -> Result {
        let api_url = self.api_url(output.as_ref().and_then(|o| o.locale.as_ref()))?;
        let url = api_url.join(&format!("product/{}", barcode))?;
        let params = output.map(|o| o.params(&["fields"]));
        self.get(url, params.as_ref())
    }

    pub(crate) fn new(v: V, locale: Locale, client: HttpClient) -> Self {
        Self { v, locale, client }
    }
}

impl OffClient<V0> {
    /// Returns the query builder for API V0.
    pub fn query(&self) -> SearchQueryV0 {
        SearchQueryV0::new()
    }

    /// Sends the given search query.
    pub fn search(&self, query: SearchQueryV0, output: Option<Output>) -> Result {
        SearchQueryV0::search(query, self, output)
    }
}

impl SearchUrl for OffClient<V0> {
    /// Returns the API V0 search URL.
    ///
    /// `https://{locale}.openfoodfacts.org/cgi/search.pl`
    fn search_url(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError> {
        let cgi_url = self.cgi_url(locale)?;
        cgi_url.join("search.pl")
    }
}

impl OffClient<V2> {
    /// Returns the query builder for API V2.
    pub fn query(&self) -> SearchQueryV2 {
        SearchQueryV2::new()
    }

    /// Sends the search query.
    pub fn search(&self, query: SearchQueryV2, output: Option<Output>) -> Result {
        SearchQueryV2::search(query, self, output)
    }

    /// Gets the products given in the `barcodes` list as a string of comma-separated
    /// product barcodes.
    ///
    /// # OFF API request
    ///
    /// `GET https://{locale}.openfoodfacts.org/api/v2/search?code=<code>,<code>,..`
    ///
    /// TODO: Support iterator (FromIter ?)
    pub fn products(&self, barcodes: &str, output: Option<Output>) -> Result {
        // Borrow output and extract Option<&Locale>
        let url = self.search_url(output.as_ref().and_then(|o| o.locale.as_ref()))?;
        let mut params = Params::new();
        params.push(("code", String::from(barcodes)));
        if let Some(output_params) = output.map(|o| o.params(&["fields"])) {
            params.extend(output_params);
        }
        self.get(url, Some(&params))
    }
}

impl SearchUrl for OffClient<V2> {
    /// Returns the API V2 search URL.
    ///
    /// `https://{locale}.openfoodfacts.org/api/v2/search`
    fn search_url(&self, locale: Option<&Locale>) -> std::result::Result<Url, ParseError> {
        // Return the API URL with the locale given in Output::locale.
        let api_url = self.api_url(locale)?;
        api_url.join("search")
    }
}

#[cfg(test)]
mod tests_client {
    use super::*;

    #[test]
    fn version() {
        let client_v0 = crate::v0().build().unwrap();
        assert_eq!(client_v0.version(), "v0");

        let client_v2 = crate::v2().build().unwrap();
        assert_eq!(client_v2.version(), "v2");
    }

    #[test]
    fn base_url_default() {
        let client = crate::v0().build().unwrap();
        assert_eq!(
            client.base_url(None).unwrap().as_str(),
            "https://world.openfoodfacts.org/"
        );
    }

    #[test]
    fn base_url_locale() {
        let client = crate::v0().build().unwrap();
        assert_eq!(
            client
                .base_url(Some(&Locale::new("gr", None)))
                .unwrap()
                .as_str(),
            "https://gr.openfoodfacts.org/"
        );
    }

    #[test]
    fn base_url_world() {
        let client = crate::v0().locale(Locale::new("gr", None)).build().unwrap();
        assert_eq!(
            client.base_url_world().unwrap().as_str(),
            "https://world.openfoodfacts.org/"
        );
    }

    #[test]
    fn client_cgi_url() {
        let client = crate::v0().build().unwrap();
        assert_eq!(
            client
                .cgi_url(Some(&Locale::new("gr", None)))
                .unwrap()
                .as_str(),
            "https://gr.openfoodfacts.org/cgi/"
        );
    }
}

#[cfg(test)]
mod tests_client_v0 {
    use super::*;

    #[test]
    fn api_url() {
        let client = crate::v0().build().unwrap();
        assert_eq!(
            client.api_url(None).unwrap().as_str(),
            "https://world.openfoodfacts.org/api/v0/"
        );
    }

    #[test]
    fn search_url() {
        let client = crate::v0().build().unwrap();
        assert_eq!(
            client
                .search_url(Some(&Locale::new("gr", None)))
                .unwrap()
                .as_str(),
            "https://gr.openfoodfacts.org/cgi/search.pl"
        );
    }
}

#[cfg(test)]
mod tests_client_v2 {
    use super::*;

    #[test]
    fn api_url() {
        let client = crate::v2().build().unwrap();
        assert_eq!(
            client.api_url(None).unwrap().as_str(),
            "https://world.openfoodfacts.org/api/v2/"
        );
    }

    #[test]
    fn search_url() {
        let client = crate::v2().build().unwrap();
        assert_eq!(
            client
                .search_url(Some(&Locale::new("gr", None)))
                .unwrap()
                .as_str(),
            "https://gr.openfoodfacts.org/api/v2/search"
        );
    }
}
