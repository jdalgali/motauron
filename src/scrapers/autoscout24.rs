// autoscout24.ch / motoscout24.ch scraper — currently disabled
//
// The REST API (api.autoscout24.ch) requires SMG B2B credentials.
// The web frontend is protected by Cloudflare Turnstile which blocks headless browsers.
//
// To enable:
//   1. Obtain API credentials from SMG Automotive (https://b2b.autoscout24.ch)
//   2. Set AS24_CLIENT_ID and AS24_CLIENT_SECRET environment variables
//   3. Uncomment the as24_categories block in main.rs
//   4. Implement the REST API client here (see git history for the previous implementation)
