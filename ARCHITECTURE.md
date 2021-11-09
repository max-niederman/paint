# Architecture

Paint consists of the following major components:

## `/web` - Web Client - "Varnish"

Provides a visual user interface to Paint.

## `/api` - API Gateway - "Oil"

Serves all requisite data to client components. It does not keep track of this state, instead relaying requests to Canvas's API and to **Pigment**.

## `/res` - Resources Service - "Pigment"

Resolves queries about Canvas resources, including **Assignments** and **Announcements**. This is achieved by caching and indexing periodically polled Canvas data.
