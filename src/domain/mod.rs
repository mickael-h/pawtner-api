//! Core marketplace domain types and enums shared across handlers and persistence.

pub mod marketplace;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Merchant,
    Client,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Merchant => "merchant",
            UserRole::Client => "client",
        }
    }
}

impl FromStr for UserRole {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "merchant" => Ok(UserRole::Merchant),
            "client" => Ok(UserRole::Client),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OfferStatus {
    Draft,
    Published,
    Archived,
}

impl OfferStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OfferStatus::Draft => "draft",
            OfferStatus::Published => "published",
            OfferStatus::Archived => "archived",
        }
    }
}

impl FromStr for OfferStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(OfferStatus::Draft),
            "published" => Ok(OfferStatus::Published),
            "archived" => Ok(OfferStatus::Archived),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListingType {
    Sale,
    Stud,
}

impl ListingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ListingType::Sale => "sale",
            ListingType::Stud => "stud",
        }
    }
}

impl FromStr for ListingType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sale" => Ok(ListingType::Sale),
            "stud" => Ok(ListingType::Stud),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimalType {
    Dog,
    Cat,
    Horse,
}

impl AnimalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnimalType::Dog => "dog",
            AnimalType::Cat => "cat",
            AnimalType::Horse => "horse",
        }
    }
}

impl FromStr for AnimalType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dog" => Ok(AnimalType::Dog),
            "cat" => Ok(AnimalType::Cat),
            "horse" => Ok(AnimalType::Horse),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Completed,
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Confirmed => "confirmed",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
        }
    }
}

impl FromStr for OrderStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(OrderStatus::Pending),
            "confirmed" => Ok(OrderStatus::Confirmed),
            "completed" => Ok(OrderStatus::Completed),
            "cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CycleStatus {
    Rest,
    Heat,
    Pregnancy,
    Nursing,
}

impl CycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CycleStatus::Rest => "rest",
            CycleStatus::Heat => "heat",
            CycleStatus::Pregnancy => "pregnancy",
            CycleStatus::Nursing => "nursing",
        }
    }
}

impl FromStr for CycleStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rest" => Ok(CycleStatus::Rest),
            "heat" => Ok(CycleStatus::Heat),
            "pregnancy" => Ok(CycleStatus::Pregnancy),
            "nursing" => Ok(CycleStatus::Nursing),
            _ => Err(()),
        }
    }
}

pub fn validate_uuid(value: &str, field: &str) -> Result<(), ApiError> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| ApiError::BadRequest(format!("invalid {}: expected UUID", field)))
}

pub fn validate_birth_date(value: &str, field: &str) -> Result<(), ApiError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        ApiError::BadRequest(format!(
            "invalid {}: expected date format YYYY-MM-DD",
            field
        ))
    })?;
    Ok(())
}

pub fn validate_animal_type(value: &str, field: &str) -> Result<(), ApiError> {
    AnimalType::from_str(value).map_err(|_| {
        ApiError::BadRequest(format!(
            "invalid {}: expected one of dog, cat, horse",
            field
        ))
    })?;
    Ok(())
}

pub fn validate_listing_type(value: &str, field: &str) -> Result<(), ApiError> {
    ListingType::from_str(value).map_err(|_| {
        ApiError::BadRequest(format!("invalid {}: expected one of sale, stud", field))
    })?;
    Ok(())
}

pub fn validate_offer_status(value: &str, field: &str) -> Result<(), ApiError> {
    OfferStatus::from_str(value).map_err(|_| {
        ApiError::BadRequest(format!(
            "invalid {}: expected one of draft, published, archived",
            field
        ))
    })?;
    Ok(())
}

pub fn validate_cycle_status(value: &str, field: &str) -> Result<(), ApiError> {
    CycleStatus::from_str(value).map_err(|_| {
        ApiError::BadRequest(format!(
            "invalid {}: expected one of rest, heat, pregnancy, nursing",
            field
        ))
    })?;
    Ok(())
}

pub fn validate_gender(value: &str, field: &str) -> Result<(), ApiError> {
    match value {
        "M" | "F" => Ok(()),
        _ => Err(ApiError::BadRequest(format!(
            "invalid {}: expected one of M, F",
            field
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_role_roundtrip() {
        assert_eq!(UserRole::from_str("merchant").unwrap().as_str(), "merchant");
        assert_eq!(UserRole::from_str("client").unwrap().as_str(), "client");
        assert!(UserRole::from_str("admin").is_err());
    }

    #[test]
    fn offer_status_roundtrip() {
        assert_eq!(OfferStatus::from_str("draft").unwrap().as_str(), "draft");
        assert_eq!(
            OfferStatus::from_str("published").unwrap().as_str(),
            "published"
        );
        assert_eq!(
            OfferStatus::from_str("archived").unwrap().as_str(),
            "archived"
        );
        assert!(OfferStatus::from_str("deleted").is_err());
    }

    #[test]
    fn listing_type_roundtrip() {
        assert_eq!(ListingType::from_str("sale").unwrap().as_str(), "sale");
        assert_eq!(ListingType::from_str("stud").unwrap().as_str(), "stud");
        assert!(ListingType::from_str("lease").is_err());
    }

    #[test]
    fn animal_type_roundtrip() {
        assert_eq!(AnimalType::from_str("dog").unwrap().as_str(), "dog");
        assert_eq!(AnimalType::from_str("cat").unwrap().as_str(), "cat");
        assert_eq!(AnimalType::from_str("horse").unwrap().as_str(), "horse");
        assert!(AnimalType::from_str("bird").is_err());
    }

    #[test]
    fn order_status_roundtrip() {
        assert_eq!(
            OrderStatus::from_str("pending").unwrap().as_str(),
            "pending"
        );
        assert_eq!(
            OrderStatus::from_str("confirmed").unwrap().as_str(),
            "confirmed"
        );
        assert_eq!(
            OrderStatus::from_str("completed").unwrap().as_str(),
            "completed"
        );
        assert_eq!(
            OrderStatus::from_str("cancelled").unwrap().as_str(),
            "cancelled"
        );
        assert!(OrderStatus::from_str("refunded").is_err());
    }

    #[test]
    fn cycle_status_roundtrip() {
        assert_eq!(CycleStatus::from_str("rest").unwrap().as_str(), "rest");
        assert_eq!(CycleStatus::from_str("heat").unwrap().as_str(), "heat");
        assert_eq!(
            CycleStatus::from_str("pregnancy").unwrap().as_str(),
            "pregnancy"
        );
        assert_eq!(
            CycleStatus::from_str("nursing").unwrap().as_str(),
            "nursing"
        );
        assert!(CycleStatus::from_str("unknown").is_err());
    }
}
