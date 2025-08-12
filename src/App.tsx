import DashboardPage from "./app/dashboard/page";
import { ThemeProvider } from "@/components/theme-provider";
import "./App.css";

function App() {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>
      <DashboardPage />
    </ThemeProvider>
  );
}

export default App;
