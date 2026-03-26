import { Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import LeaderboardPage from "./pages/LeaderboardPage";
import ComparePage from "./pages/ComparePage";
import RunDetailPage from "./pages/RunDetailPage";
import AboutPage from "./pages/AboutPage";

export default function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<LeaderboardPage />} />
        <Route path="/compare" element={<ComparePage />} />
        <Route path="/run/:runId" element={<RunDetailPage />} />
        <Route path="/about" element={<AboutPage />} />
      </Routes>
    </Layout>
  );
}
