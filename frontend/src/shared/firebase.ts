import { initializeApp } from "firebase/app";
import { getFirestore } from "firebase/firestore";

const firebaseConfig = {
  apiKey: "AIzaSyAIImrEfoH6GKtDQqq28rIVGwNslsv26Ug",
  authDomain: "motauron-ch.firebaseapp.com",
  projectId: "motauron-ch",
  storageBucket: "motauron-ch.firebasestorage.app",
  messagingSenderId: "1022799380090",
  appId: "1:1022799380090:web:fc7a2c1d038b34d904c5c3",
  measurementId: "G-F7QF49WC9R"
};

export const app = initializeApp(firebaseConfig);
export const db = getFirestore(app);
