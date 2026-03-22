import React from "react";
import { ListingsFeed } from "../features/listings/ListingsFeed";

export const DashboardPage: React.FC = () => {
  return (
    <div className="page-container">
      <header className="dashboard-header">
        <h1 className="header-title">Market Pulse</h1>
        <p className="header-subtitle">Live analytics for the Swiss motorcycle market.</p>
      </header>
      
      <main>
        <ListingsFeed />
      </main>
    </div>
  );
};
