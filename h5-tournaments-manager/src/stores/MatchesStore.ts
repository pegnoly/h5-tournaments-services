import { create } from "zustand"
import { Match } from "../common/types"
import { invoke } from "@tauri-apps/api/core"

type MatchesStoreData = {
    matches: Match[]
}

type MatchesStoreActions = {
    load: (tournament_id: string) => void,
    update: (match: Match) => void
}

export const useMatchesStore = create<MatchesStoreData & MatchesStoreActions>((set, get) => ({
    matches: [],
    async load(tournament_id) {
        await invoke("load_matches", {tournamentId: tournament_id})
            .then((matches_data) => set({matches: matches_data as Match[]}))
    },
    async update(match) {
        await invoke("update_match", {matchToUpdate: match});
        const updatedMatches = get().matches.map((m) => {
            if (m.id == match.id) {
                return match;
            }
            else {
                return m;
            }
        })
        set({matches: updatedMatches});
    },
}))