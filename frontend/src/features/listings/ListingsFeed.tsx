import React, { useState, useMemo } from "react";
import { useListings } from "./useListings";
import { DealCard } from "./DealCard";
import "./ListingsFeed.css";

type SortKey = "score" | "price-asc" | "price-desc" | "mileage" | "year";

export const ListingsFeed: React.FC = () => {
  const { listings, loading, scrape, scraping } = useListings();

  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState<SortKey>("score");
  const [maxPrice, setMaxPrice] = useState("");
  const [maxMileage, setMaxMileage] = useState("");
  const [yearFrom, setYearFrom] = useState("");
  const [yearTo, setYearTo] = useState("");
  const [sellerType, setSellerType] = useState<"all" | "private" | "dealer">("all");

  const filtered = useMemo(() => {
    const term = search.trim().toLowerCase();
    const priceMax = maxPrice ? parseInt(maxPrice) : Infinity;
    const mileMax = maxMileage ? parseInt(maxMileage) : Infinity;
    const yFrom = yearFrom ? parseInt(yearFrom) : 0;
    const yTo = yearTo ? parseInt(yearTo) : 9999;

    const base = listings.filter((l) => {
      if (term && !l.title.toLowerCase().includes(term)) return false;
      if (l.price_chf > priceMax) return false;
      if (l.mileage_km > mileMax) return false;
      if (l.year < yFrom || l.year > yTo) return false;
      if (sellerType === "private" && !l.is_private) return false;
      if (sellerType === "dealer" && l.is_private) return false;
      return true;
    });

    const copy = [...base];
    switch (sortBy) {
      case "price-asc":  return copy.sort((a, b) => a.price_chf - b.price_chf);
      case "price-desc": return copy.sort((a, b) => b.price_chf - a.price_chf);
      case "mileage":    return copy.sort((a, b) => a.mileage_km - b.mileage_km);
      case "year":       return copy.sort((a, b) => b.year - a.year);
      default:           return copy.sort((a, b) => b.price_score - a.price_score);
    }
  }, [listings, search, sortBy, maxPrice, maxMileage, yearFrom, yearTo, sellerType]);

  if (loading) {
    return (
      <div className="feed-loading">
        <div className="spinner"></div>
        <p>Syncing market data…</p>
      </div>
    );
  }

  if (listings.length === 0) {
    return (
      <div className="feed-empty glass-panel">
        <h2>No listings found</h2>
        {scrape ? (
          <p>
            No local data yet.{" "}
            <button className="scrape-btn" onClick={scrape} disabled={scraping}>
              {scraping ? "Scraping…" : "Scrape now"}
            </button>
          </p>
        ) : (
          <p>The agent hasn't indexed any active deals yet. Check back later.</p>
        )}
      </div>
    );
  }

  return (
    <>
      <div className="filter-bar">
        <div className="filter-row">
          <input
            className="filter-search"
            type="text"
            placeholder="Search brand / model…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />

          <select
            className="sort-select"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as SortKey)}
          >
            <option value="score">Best Deal</option>
            <option value="price-asc">Price ↑</option>
            <option value="price-desc">Price ↓</option>
            <option value="mileage">Lowest Mileage</option>
            <option value="year">Newest First</option>
          </select>

          {scrape && (
            <button
              className="scrape-btn"
              onClick={scrape}
              disabled={scraping}
              title="Pull fresh data from motorradhandel.ch"
            >
              {scraping ? "Scraping…" : "↻ Refresh"}
            </button>
          )}
        </div>

        <div className="filter-row filter-row--secondary">
          <label className="filter-label">
            Max CHF
            <input
              className="filter-number"
              type="number"
              placeholder="e.g. 12000"
              value={maxPrice}
              onChange={(e) => setMaxPrice(e.target.value)}
            />
          </label>

          <label className="filter-label">
            Max km
            <input
              className="filter-number"
              type="number"
              placeholder="e.g. 30000"
              value={maxMileage}
              onChange={(e) => setMaxMileage(e.target.value)}
            />
          </label>

          <label className="filter-label">
            Year from
            <input
              className="filter-number"
              type="number"
              placeholder="e.g. 2020"
              value={yearFrom}
              onChange={(e) => setYearFrom(e.target.value)}
            />
          </label>

          <label className="filter-label">
            Year to
            <input
              className="filter-number"
              type="number"
              placeholder="e.g. 2024"
              value={yearTo}
              onChange={(e) => setYearTo(e.target.value)}
            />
          </label>

          <div className="filter-label">
            Seller
            <div className="seller-toggle">
              {(["all", "private", "dealer"] as const).map((v) => (
                <button
                  key={v}
                  className={`toggle-btn${sellerType === v ? " active" : ""}`}
                  onClick={() => setSellerType(v)}
                >
                  {v.charAt(0).toUpperCase() + v.slice(1)}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="filter-result-count">
          {filtered.length} of {listings.length} listings
        </div>
      </div>

      <div className="listings-grid">
        {filtered.map((listing) => (
          <DealCard key={listing.listing_id} listing={listing} />
        ))}
      </div>
    </>
  );
};
