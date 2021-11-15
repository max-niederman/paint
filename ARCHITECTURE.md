# Architecture

Paint consists of the following major components:

## `/web` - Web Client - "Varnish"

Provides a visual user interface to Paint.

## `/api` - API Gateway - "Oil"

Serves all requisite data to client components. It does not keep track of this state, instead relaying requests to Canvas's API and to **Pigment**.

## `/res` - Resources Cache - "Pigment"

Read-through cache of Canvas resources, including **Assignments** and **Announcements**. This is necessary to improve query performance.

## `/canvas` - Canvas API Client

Rust client for Canvas's Rest and GraphQL API. GraphQL is used whenever possible to prevent overquerying, but we fallback to REST where the GraphQL API is incomplete.