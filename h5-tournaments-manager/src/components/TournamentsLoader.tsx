import { Button, Select } from "antd";
import { useTournamentsStore } from "../stores/TournamentsStore";
import { useMatchesStore } from "../stores/MatchesStore";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useHeroesStore } from "../stores/HeroesStore";

export function TournamentsLoader() {
    const tournaments = useTournamentsStore((state) => state.tournaments)
    const loadHeroes = useHeroesStore((state) => state.load)
    const loadMatches = useMatchesStore((state) => state.load)

    const [tournamentId, setTournamentId] = useState<string>("")

    async function selectTournament() {
        const selectedTournamentType = tournaments.find((t) => t.id == tournamentId)!.mod_type
        loadHeroes(selectedTournamentType)
        loadMatches(tournamentId)
    }

    return (
        <div style={{width: '50%', display: 'flex', flexDirection: 'row', alignContent: 'center'}}>
            <Select
                onChange={(e) => setTournamentId(e)}
                style={{width: '50%'}}>{tournaments.map((t, i) => (
                <Select.Option key={i} value={t.id}>{t.name}</Select.Option>
            ))}</Select>
            <div style={{paddingLeft: 10}}>
                <Button
                    onClick={selectTournament}
                >Загрузить данные турнира</Button>
            </div>
            <div style={{paddingLeft: 10}}>
                <Button
                    onClick={() => {invoke("load_games_for_stats", {tournamentId: tournamentId})}}
                >Собрать статистику турнира</Button>
            </div>
        </div>
    )
}