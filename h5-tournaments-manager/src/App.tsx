import { useEffect, useState } from "react";
import { useTournamentsStore } from "./stores/TournamentsStore";
import { TournamentsLoader } from "./components/TournamentsLoader";
import { MatchesList } from "./components/MatchesList";
import { useRacesStore } from "./stores/RacesStore";

enum AppState {
  Ready,
  NotReady
}

function App() {
  const loadTournaments = useTournamentsStore((state) => state.load);
  const loadRaces = useRacesStore((state) => state.load);

  const [appState, setAppState] = useState<AppState>(AppState.NotReady);

  useEffect(() => {
    if (appState == AppState.Ready) {
      loadTournaments();
      // loadHeroes();
      loadRaces();
    }
  }, [appState])

  if (appState == AppState.NotReady) {
    setAppState(AppState.Ready);
  }

  return (
    <div>
      <TournamentsLoader/>
      <MatchesList/>
    </div>
  );
}

export default App;
