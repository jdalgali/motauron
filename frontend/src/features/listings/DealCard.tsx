import React from "react";
import { MotorcycleListing } from "../../shared/types";
import "./DealCard.css"; // We'll create a module-like CSS or shared CSS inside features soon!

interface Props {
  listing: MotorcycleListing;
}

export const DealCard: React.FC<Props> = ({ listing }) => {
  // Map price label to a specific color semantic
  const getScoreColor = (score: number) => {
    if (score >= 15) return "var(--success)";
    if (score >= 7) return "var(--success)";
    if (score >= -6 && score <= 6) return "var(--text-secondary)";
    if (score <= -15) return "var(--danger)";
    return "var(--warning)";
  };

  const isPositive = listing.price_score > 0;
  const sign = isPositive ? "+" : "";

  return (
    <a href={listing.url} target="_blank" rel="noopener noreferrer" className="deal-card glass-panel">
      {listing.image_url && (
        <div className="card-image">
          <img
            src={listing.image_url}
            alt={listing.title}
            loading="lazy"
            referrerPolicy="no-referrer"
            onError={(e) => {
              const parent = (e.target as HTMLElement).parentElement;
              if (parent) parent.style.display = "none";
            }}
          />
        </div>
      )}
      <div className="card-header">
        <h3 className="car-title">{listing.title}</h3>
        <span className="car-year">{listing.year}</span>
      </div>
      {listing.generation && (
        <div className="card-generation">{listing.generation}</div>
      )}
      
      <div className="card-body">
        <div className="card-price">
          CHF {listing.price_chf.toLocaleString()}
          {listing.original_price_chf && listing.original_price_chf > listing.price_chf && (
            <span className="price-drop">
              (was {listing.original_price_chf.toLocaleString()})
            </span>
          )}
        </div>
        
        <div className="card-metrics">
          <div className="metric">
            <span className="metric-icon">🛣️</span>
            {listing.mileage_km.toLocaleString()} km
          </div>
          <div className="metric">
            <span className="metric-icon">📍</span>
            {listing.location} ({listing.kanton})
          </div>
          <div className="metric">
            <span className="metric-icon">👤</span>
            {listing.is_private ? "Private Seller" : listing.seller_name}
          </div>
        </div>
      </div>

      <div className="card-footer" style={{ borderTopColor: getScoreColor(listing.price_score) }}>
        <div className="score-badge" style={{ color: getScoreColor(listing.price_score) }}>
          <strong>{listing.price_label}</strong>
          <span className="score-number">
             {sign}{listing.price_score}%
          </span>
        </div>
        <div className="peers">vs {listing.score_peers} peers</div>
      </div>
    </a>
  );
};
