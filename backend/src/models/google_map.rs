use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GoogleMapSearchParams {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GoogleMapPayload {
    #[serde(rename = "textQuery")]
    text_query: String,
    #[serde(rename = "languageCode")]
    language_code: String,
    #[serde(rename = "locationBias")]
    location_bias: Option<GoogleMapLocationBias>,
    #[serde(rename = "pageSize")]
    page_size: Option<u8>,
}

impl GoogleMapPayload {
    pub fn new(
        text_query: String,
        language_code: String,
        latitude: f64,
        longitude: f64,
        radius: f64,
        page_size: Option<u8>,
    ) -> Self {
        Self {
            text_query,
            language_code,
            location_bias: Some(GoogleMapLocationBias {
                circle: GoogleMapLocationBiasCircle {
                    center: PlaceLocation {
                        latitude,
                        longitude,
                    },
                    radius,
                },
            }),
            page_size: Some(page_size.unwrap_or_else(|| 20)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GoogleMapLocationBias {
    circle: GoogleMapLocationBiasCircle,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GoogleMapLocationBiasCircle {
    center: PlaceLocation,
    radius: f64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GoogleMapResponse {
    #[serde(default)]
    places: Vec<Place>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PlaceLocation {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PlaceDisplayName {
    text: String,
    #[serde(rename(deserialize = "languageCode"))]
    language_code: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct Place {
    id: String,
    #[serde(rename(deserialize = "formattedAddress"))]
    formatted_address: String,
    location: PlaceLocation,
    #[serde(rename(deserialize = "displayName"))]
    display_name: PlaceDisplayName,
}
