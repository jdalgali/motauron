import { useEffect, useState } from "react";
import { collection, onSnapshot, query, where, orderBy } from "firebase/firestore";
import { db } from "../../shared/firebase";
import { MotorcycleListing } from "../../shared/types";

export function useListings() {
  const [listings, setListings] = useState<MotorcycleListing[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // We only want active listings.
    // We sort locally to avoid needing a Firebase Composite Index!
    const q = query(
      collection(db, "listings"),
      where("status", "==", "active")
    );

    const unsubscribe = onSnapshot(q, (snapshot) => {
      const data: MotorcycleListing[] = [];
      snapshot.forEach((doc) => {
        data.push(doc.data() as MotorcycleListing);
      });
      
      // Sort locally: Best deals (highest score) first
      data.sort((a, b) => b.price_score - a.price_score);
      
      setListings(data);
      setLoading(false);
    }, (error) => {
      console.error("Firebase Error:", error);
      setLoading(false);
    });

    return () => unsubscribe();
  }, []);

  return { listings, loading };
}
