export type ListingStatus = "active" | "sold" | "relisted";

export interface MotorcycleListing {
  listing_id: number;
  fingerprint: string;
  status: ListingStatus;
  first_seen: string;
  last_seen: string;
  category: string;
  title: string;
  price_chf: number;
  original_price_chf?: number;
  year: number;
  mileage_km: number;
  url: string;
  previous_listing_id?: number | null;
  location: string;
  kanton: string;
  is_private: boolean;
  seller_name: string;
  price_score: number;
  price_label: string;
  score_peers: number;
}
