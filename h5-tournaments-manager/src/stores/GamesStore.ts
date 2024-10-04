import { create } from "zustand"
import { Game } from "../common/types"
import { invoke } from "@tauri-apps/api/core"

type GamesStoreData = {
    games: Game[]
}

type GamesStoreActions = {
    load: (match_id: string) => Game[],
    update: (game: Game) => void,
    add: (match_id: string) => void
}

export const useGamesStore = create<GamesStoreData & GamesStoreActions>((set, get) => ({
    games: [],
    load(match_id) {
        let games: Game[] = []
        invoke("load_games", {matchId: match_id})
            .then((games_data) => {
                console.log("Games data fetched with match id ", match_id, ": ", games_data);
                games = games_data as Game[];
                console.log("Games here: ", games);
            });
        return games;
    },
    async update(game) {
        await invoke("update_game", {game: game});
        const updatedGames = get().games.map((g) => {
            if (g.id == game.id) {
                return game;
            }
            else {
                return g;
            }
        })
        set({games: updatedGames});
    },
    async add(match_id) {
        await invoke("create_game", {matchId: match_id});
    },
}))