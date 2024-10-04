import { useState } from "react";
import { BargainsColor, Game, HeroType, RaceType } from "../common/types";
import { useGamesStore } from "../stores/GamesStore";
import { useHeroesStore } from "../stores/HeroesStore";
import { useRacesStore } from "../stores/RacesStore";
import { Select, Typography } from "antd";

interface GameRendererSchema {
    game: Game
}

export function GameRenderer(schema: GameRendererSchema) {
    const heroes = useHeroesStore((state) => state.heroes);
    const races = useRacesStore((state) => state.races);
    const updateGame = useGamesStore((state) => state.update);

    console.log("Game: ", schema.game);

    const [firstPlayerRace, setFirstPlayerRace] = useState<RaceType | undefined>(schema.game.first_player_race);
    const [firstPlayerHero, setFirstPlayerHero] = useState<HeroType | undefined>(schema.game.first_player_hero);
    const [secondPlayerRace, setSecondPlayerRace] = useState<RaceType | undefined>(schema.game.second_player_race);
    const [secondPlayerHero, setSecondPlayerHero] = useState<HeroType | undefined>(schema.game.second_player_hero);

    const [bargainsColor, setBargainsColor] = useState<BargainsColor | undefined>(schema.game.bargains_color);
    const [bargainsAmount, setBargainsAmount] = useState<number>(schema.game.bargains_amount);

    function updateFirstPlayerRace(race: RaceType) {
        setFirstPlayerRace(race);
        updateGame({
            ...schema.game,
            first_player_race: race
        });
    }

    function updateFirstPlayerHero(hero: HeroType) {
        setFirstPlayerHero(hero);
        updateGame({
            ...schema.game,
            first_player_hero: hero
        });
    }

    function updateSecondPlayerRace(race: RaceType) {
        setSecondPlayerRace(race);
        updateGame({
            ...schema.game,
            second_player_race: race
        });
    }

    function updateSecondPlayerHero(hero: HeroType) {
        setSecondPlayerHero(hero);
        updateGame({
            ...schema.game,
            second_player_hero: hero
        });
    }

    function updateBargainsColor(color: BargainsColor) {
        setBargainsColor(color);
        updateGame({
            ...schema.game,
            bargains_color: color
        });
    }

    function updateBargainsAmount(amount: number) {
        setBargainsAmount(amount);
        updateGame({
            ...schema.game,
            bargains_amount: amount
        });
    }
    
    return (
        <div style={{height: 120, display: 'flex', flexDirection: 'column', alignItems: "center", gap: 5}}>
            <div style={{width: '100%', display: 'flex', flexDirection: 'row', justifyContent: "center", gap: 20}}>
                <Select onChange={(v) => updateFirstPlayerRace(v)} value={firstPlayerRace}>
                    <Select.Option key={-1} value={RaceType.NotDetected}>Не определено</Select.Option>
                    {races.map((r, i) => (
                        <Select.Option key={i} value={r.id}>{r.actual_name}</Select.Option>
                    ))}
                </Select>
                <Typography.Text>VS</Typography.Text>
                <Select onChange={(v) => updateSecondPlayerRace(v)} value={secondPlayerRace}>
                    <Select.Option key={-1} value={RaceType.NotDetected}>Не определено</Select.Option>
                    {races.map((r, i) => (
                        <Select.Option key={i} value={r.id}>{r.actual_name}</Select.Option>
                    ))}
                </Select>
            </div>
            <div style={{width: '100%', display: 'flex', flexDirection: 'row', justifyContent: "center", gap: 20}}>
                <Select onChange={(v) => updateFirstPlayerHero(v)} value={firstPlayerHero}>
                    <Select.Option key={-1} value={HeroType.NotDetected}>Не определено</Select.Option>
                    {heroes.map((h, i) => (
                        <Select.Option key={i} value={h.id}>{h.actual_name}</Select.Option>
                    ))}
                </Select>
                <Typography.Text>VS</Typography.Text>
                <Select onChange={(v) => updateSecondPlayerHero(v)} value={secondPlayerHero}>
                    <Select.Option key={-1} value={HeroType.NotDetected}>Не определено</Select.Option>
                    {heroes.map((h, i) => (
                        <Select.Option key={i} value={h.id}>{h.actual_name}</Select.Option>
                    ))}
                </Select>
            </div>
            <div style={{width: '100%', display: 'flex', flexDirection: 'row', justifyContent: "center", gap: 20}}>
                <Typography.Text>Торг:</Typography.Text>
                <Select onChange={(v) => updateBargainsColor(v)} value={bargainsColor}>
                    <Select.Option key={0} value={BargainsColor.NotDetected}>Не определено</Select.Option>
                    <Select.Option key={1} value={BargainsColor.Red}>Красный</Select.Option>
                    <Select.Option key={2} value={BargainsColor.Blue}>Синий</Select.Option>
                </Select>
                <Typography.Text style={{textAlign: 'center'}} editable={{onChange(value) {
                    updateBargainsAmount(parseInt(value))
                },}}>{bargainsAmount}</Typography.Text>
            </div>
        </div>
    )
}