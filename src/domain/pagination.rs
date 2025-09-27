//! Pagination types for API responses
//!
//! Common types for handling paginated responses

use serde::{Deserialize, Serialize};

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The current page of results
    pub data: Vec<T>,

    /// Pagination metadata
    pub meta: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number (1-indexed)
    pub page: usize,

    /// Number of items per page
    pub limit: usize,

    /// Total number of items
    pub total: usize,

    /// Total number of pages
    pub total_pages: usize,

    /// Whether there is a next page
    pub has_next: bool,

    /// Whether there is a previous page
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(items: Vec<T>, page: usize, limit: usize, total: usize) -> Self {
        let total_pages = if limit > 0 {
            (total + limit - 1) / limit // Ceiling division
        } else {
            1
        };

        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            data: items,
            meta: PaginationMeta {
                page,
                limit,
                total,
                total_pages,
                has_next,
                has_prev,
            }
        }
    }

    /// Create a paginated response from a full list
    pub fn from_full_list(all_items: Vec<T>, page: usize, limit: usize) -> Self {
        let total = all_items.len();

        // Default to page 1 if invalid
        let page = if page == 0 { 1 } else { page };

        // Default to 20 items per page if invalid
        let limit = if limit == 0 { 20 } else { limit };

        // Calculate the range of items to return
        let skip = (page - 1) * limit;
        let items: Vec<T> = all_items
            .into_iter()
            .skip(skip)
            .take(limit)
            .collect();

        Self::new(items, page, limit, total)
    }
}

/// Request parameters for pagination
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    pub page: Option<usize>,

    /// Items per page
    pub limit: Option<usize>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

impl PaginationParams {
    /// Get the page number (defaults to 1)
    pub fn get_page(&self) -> usize {
        self.page.unwrap_or(1).max(1)
    }

    /// Get the limit (defaults to 20, max 100)
    pub fn get_limit(&self) -> usize {
        self.limit.unwrap_or(20).min(100).max(1)
    }
}