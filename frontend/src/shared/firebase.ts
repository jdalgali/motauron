import { initializeApp } from "firebase/app";
import { getFirestore, Firestore } from "firebase/firestore";

// Firebase is only initialised when a key is present.
// In local-dev mode (VITE_API_URL set), no key is needed and db will be null.
const apiKey = import.meta.env.VITE_FIREBASE_API_KEY as string | undefined;

let db: Firestore | null = null;

if (apiKey) {
  const app = initializeApp({
    apiKey,
    authDomain: "motauron-ch.firebaseapp.com",
    projectId: "motauron-ch",
    storageBucket: "motauron-ch.firebasestorage.app",
    messagingSenderId: "1022799380090",
    appId: "1:1022799380090:web:fc7a2c1d038b34d904c5c3",
  });
  db = getFirestore(app);
}

export { db };
