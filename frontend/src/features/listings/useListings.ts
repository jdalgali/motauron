import { useEffect, useState, useCallback } from "react";
import { collection, onSnapshot, query, where } from "firebase/firestore";
import { db } from "../../shared/firebase";
import { MotorcycleListing } from "../../shared/types";

const API_URL = import.meta.env.VITE_API_URL as string | undefined;

export function useListings() {
  const [listings, setListings] = useState<MotorcycleListing[]>([]);
  const [loading, setLoading] = useState(true);
  const [scraping, setScraping] = useState(false);

  const fetchFromApi = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fetch(`${API_URL}/api/listings`);
      const data: MotorcycleListing[] = await res.json();
      const active = data
        .filter((l) => l.status === "active")
        .sort((a, b) => b.price_score - a.price_score);
      setListings(active);
    } catch (e) {
      console.error("API fetch error:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const triggerScrape = useCallback(async () => {
    if (scraping) return;
    setScraping(true);
    try {
      await fetch(`${API_URL}/api/scrape`, { method: "POST" });
      await fetchFromApi();
    } catch (e) {
      console.error("Scrape error:", e);
    } finally {
      setScraping(false);
    }
  }, [scraping, fetchFromApi]);

  useEffect(() => {
    if (API_URL) {
      fetchFromApi();
      return;
    }

    // Production: Firebase real-time listener.
    if (!db) {
      console.error("No API_URL and no Firebase key — nothing to load.");
      setLoading(false);
      return;
    }

    const q = query(collection(db, "listings"), where("status", "==", "active"));
    const unsubscribe = onSnapshot(
      q,
      (snapshot) => {
        const data: MotorcycleListing[] = [];
        snapshot.forEach((doc) => data.push(doc.data() as MotorcycleListing));
        data.sort((a, b) => b.price_score - a.price_score);
        setListings(data);
        setLoading(false);
      },
      (error) => {
        console.error("Firebase error:", error);
        setLoading(false);
      }
    );

    return () => unsubscribe();
  }, [fetchFromApi]);

  return {
    listings,
    loading,
    // Only present in local-API mode.
    scrape: API_URL ? triggerScrape : undefined,
    scraping,
  };
}
