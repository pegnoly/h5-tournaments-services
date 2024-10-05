import { Button, Select } from "antd";
import { useTournamentsStore } from "../stores/TournamentsStore";
import { useMatchesStore } from "../stores/MatchesStore";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function TournamentsLoader() {
    const tournaments = useTournamentsStore((state) => state.tournaments);
    const loadMatches = useMatchesStore((state) => state.load);

    const [tournamentId, setTournamentId] = useState<string>("");

    return (
        <div style={{width: '50%', display: 'flex', flexDirection: 'row', alignContent: 'center'}}>
            <Select
                onChange={(e) => setTournamentId(e)}
                style={{width: '50%'}}>{tournaments.map((t, i) => (
                <Select.Option key={i} value={t.id}>{t.name}</Select.Option>
            ))}</Select>
            <div style={{paddingLeft: 10}}>
                <Button
                    onClick={() => loadMatches(tournamentId)}
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