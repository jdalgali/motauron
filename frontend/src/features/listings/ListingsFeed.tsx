import React from "react";
import { useListings } from "./useListings";
import { DealCard } from "./DealCard";
import "./ListingsFeed.css";

export const ListingsFeed: React.FC = () => {
  const { listings, loading } = useListings();

  if (loading) {
    return (
      <div className="feed-loading">
        <div className="spinner"></div>
        <p>Syncing market data...</p>
      </div>
    );
  }

  if (listings.length === 0) {
    return (
      <div className="feed-empty glass-panel">
        <h2>No listings found</h2>
        <p>The agent hasn't indexed any active deals yet. Check back later.</p>
      </div>
    );
  }

  return (
    <div className="listings-grid">
      {listings.map((listing) => (
        <DealCard key={listing.listing_id} listing={listing} />
      ))}
    </div>
  );
};
