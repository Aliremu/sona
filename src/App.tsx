import DashboardPage from "./app/dashboard/page";
import { ThemeProvider } from "@/components/theme-provider";
import { Toaster } from "@/components/ui/sonner";
import "./App.css";

function App() {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>
      <DashboardPage />
      <Toaster />
    </ThemeProvider>
  );
}

export default App;
