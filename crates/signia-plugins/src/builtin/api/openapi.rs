//! OpenAPI document builder for the built-in read-only API.
//!
//! This module returns a deterministic OpenAPI 3.0 JSON document describing
//! the built-in SIGNIA plugin catalog endpoints.
//!
//! Design constraints:
//! - No filesystem or network I/O.
//! - Deterministic output.
//! - Uses JSON only (no YAML dependency).

#![cfg(feature = "builtin")]

use serde_json::{json, Value};

use super::ApiResponse;

/// Return an OpenAPI 3.0 JSON document describing the built-in API.
///
/// This document is intentionally minimal but valid, and suitable for
/// code generation and interactive docs.
///
/// Endpoints:
/// - GET /v1/health
/// - GET /v1/builtin/specs
/// - GET /v1/builtin/specs/{id}
/// - GET /v1/builtin/link-graph
pub fn get_openapi_json() -> ApiResponse<Value> {
    ApiResponse {
        ok: true,
        data: openapi_doc(),
    }
}

/// Build the OpenAPI document as a deterministic JSON value.
pub fn openapi_doc() -> Value {
    // NOTE: We define schemas loosely to avoid depending on external schema crates.
    // The runtime JSON payloads are still fully deterministic and stable.
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "SIGNIA Built-in Plugin Catalog API",
            "version": "0.1.0",
            "description": "Read-only API that exposes built-in SIGNIA plugin specifications and related metadata.",
            "license": { "name": "MIT OR Apache-2.0" }
        },
        "servers": [
            { "url": "http://localhost:8787", "description": "Local development server" }
        ],
        "paths": {
            "/v1/health": {
                "get": {
                    "operationId": "health",
                    "summary": "Health check",
                    "responses": {
                        "200": {
                            "description": "OK",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ApiResponseString" }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/builtin/specs": {
                "get": {
                    "operationId": "getBuiltinSpecs",
                    "summary": "List built-in plugin specs",
                    "responses": {
                        "200": {
                            "description": "OK",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ApiResponsePluginSpecList" }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/builtin/specs/{id}": {
                "get": {
                    "operationId": "getBuiltinSpecById",
                    "summary": "Get a built-in plugin spec by id",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" },
                            "description": "Plugin id, e.g. builtin.repo"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "OK",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ApiResponsePluginSpecOptional" }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/builtin/link-graph": {
                "get": {
                    "operationId": "getBuiltinLinkGraph",
                    "summary": "Get a link graph derived from built-in plugin specs",
                    "responses": {
                        "200": {
                            "description": "OK",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ApiResponseJson" }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "ApiResponseString": {
                    "type": "object",
                    "required": ["ok", "data"],
                    "properties": {
                        "ok": { "type": "boolean" },
                        "data": { "type": "string" }
                    }
                },
                "ApiResponseJson": {
                    "type": "object",
                    "required": ["ok", "data"],
                    "properties": {
                        "ok": { "type": "boolean" },
                        "data": { "type": "object", "additionalProperties": true }
                    }
                },
                "PluginSpec": {
                    "type": "object",
                    "required": ["id", "title", "version", "supports", "limits", "wants", "meta", "description"],
                    "properties": {
                        "id": { "type": "string" },
                        "title": { "type": "string" },
                        "version": { "type": "string" },
                        "description": { "type": "string" },
                        "supports": { "type": "array", "items": { "type": "string" } },
                        "limits": { "type": "object", "additionalProperties": { "type": "integer" } },
                        "wants": { "type": "object", "additionalProperties": { "type": "boolean" } },
                        "meta": { "type": "object", "additionalProperties": { "type": "string" } }
                    }
                },
                "ApiResponsePluginSpecList": {
                    "type": "object",
                    "required": ["ok", "data"],
                    "properties": {
                        "ok": { "type": "boolean" },
                        "data": { "type": "array", "items": { "$ref": "#/components/schemas/PluginSpec" } }
                    }
                },
                "ApiResponsePluginSpecOptional": {
                    "type": "object",
                    "required": ["ok", "data"],
                    "properties": {
                        "ok": { "type": "boolean" },
                        "data": {
                            "oneOf": [
                                { "$ref": "#/components/schemas/PluginSpec" },
                                { "type": "null" }
                            ]
                        }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_is_valid_shape() {
        let doc = openapi_doc();
        assert_eq!(doc.get("openapi").and_then(|v| v.as_str()), Some("3.0.3"));
        assert!(doc.get("paths").is_some());
        assert!(doc.get("components").is_some());
    }

    #[test]
    fn endpoint_paths_exist() {
        let doc = openapi_doc();
        let paths = doc.get("paths").unwrap();
        assert!(paths.get("/v1/builtin/specs").is_some());
        assert!(paths.get("/v1/builtin/link-graph").is_some());
    }
}
