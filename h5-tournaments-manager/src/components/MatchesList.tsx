import InfiniteScroll from "react-infinite-scroll-component";
import { useGamesStore } from "../stores/GamesStore";
import { useMatchesStore } from "../stores/MatchesStore";
import { Button, Carousel, List, Select, Typography } from "antd";
import { BargainsColor, Game, HeroType, Match, RaceType } from "../common/types";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Slider from "react-slick";
import { GameRenderer } from "./GameRenderer";

// import "slick-carousel/slick/slick.css";
// import "slick-carousel/slick/slick-theme.css";

export function MatchesList() {
    const matches = useMatchesStore((state) => state.matches);
    return (
        <>
            <InfiniteScroll
                dataLength={matches.length}
                height={950}
                hasMore={false}
                next={() => {}}     
                loader={<h4>Загружается...</h4>}
            >
                <List>{matches.map((m, i) => (
                    <List.Item key={i}>
                        <MatchRenderer match={m}/>
                    </List.Item>
                ))}</List>
            </InfiniteScroll>
        </>
    )
}

interface MatchRendererSchema {
    match: Match
}

function MatchRenderer(schema: MatchRendererSchema) {
    
    const addGame = useGamesStore((state) => state.add);

    const [games, setGames] = useState<Game[]>([]);
    const updateMatch = useMatchesStore((state) => state.update);

    useEffect(() => {
        if (schema.match.id != undefined) {
            invoke("load_games", {matchId: schema.match.id})
                .then((g) => setGames(g as Game[]))
        }
    }, [schema.match.id])

    function firstPlayerUpdated(player: string) {   
        updateMatch({
            ...schema.match,
            first_player: player
        })
    }

    function secondPlayerUpdated(player: string) {
        updateMatch({
            ...schema.match,
            second_player: player
        })
    }

    return (
        <>
            <div style={{width: '100%', display: 'flex', flexDirection: 'row'}}>
                <div style={{width: '25%', display: 'flex', flexDirection: 'column', alignItems: 'center'}}>
                    <OpponentsRenderer 
                        first_player={schema.match.first_player} 
                        second_player={schema.match.second_player}
                        first_player_update_callback={firstPlayerUpdated}
                        second_player_update_callback={secondPlayerUpdated}
                    />
                    <Button style={{width: '50%'}} onClick={() => addGame(schema.match.id)}>Добавить игру</Button>
                </div>
                <GamesList games={games}/>
            </div>
        </>
    )
}

interface OpponentsRendererSchema {
    first_player: string,
    second_player: string,
    first_player_update_callback: (player: string) => void,
    second_player_update_callback: (player: string) => void
}

function OpponentsRenderer(schema: OpponentsRendererSchema) {
    
    const [firstPlayer, setFirstPlayer] = useState<string>(schema.first_player);
    const [secondPlayer, setSecondPlayer] = useState<string>(schema.second_player);

    function updateFirstPlayer(newText: string) {
        setFirstPlayer(newText);
        schema.first_player_update_callback(newText);
    }

    function updateSecondPlayer(newText: string) {
        setSecondPlayer(newText);
        schema.second_player_update_callback(newText);
    }


    return (
        <div style={{display: 'flex', flexDirection: 'column', alignItems: 'center'}}>
            <Typography.Text editable={{onChange(newText) {
                updateFirstPlayer(newText)
            }}}>{firstPlayer}</Typography.Text>
            <Typography.Text>vs</Typography.Text>
            <Typography.Text editable={{onChange(newText) {
                updateSecondPlayer(newText)
            }}}>{secondPlayer}</Typography.Text>
        </div>
    )
}

interface GamesListSchema {
    games: Game[]
}

function SampleArrow(props: any) {
    const { className, style, onClick } = props;
    return (
      <div
        className={className}
        style={{ ...style, backgroundColor: "black" }}
        onClick={onClick}
      />
    );
  }

function GamesList(schema: GamesListSchema) {
    //console.log("Games: ", schema.games);

    const settings = {
        dots: true,
        // className: "center",
        // centerMode: true,
        slidesToShow: 1,
        nextArrow: <SampleArrow/>,
        prevArrow: <SampleArrow/>
    }

    return (
        <div style={{width: '75%', alignItems: 'center'}}>
            {schema.games.length == 1 ? 
            <div style={{width: '80%', alignContent: 'center'}}><GameRenderer key={0} game={schema.games[0]}/></div> :
            <div className="slider-container" style={{width: '80%', alignContent: 'center'}}>
                <Slider {...settings}>{schema.games.map((g, i) => (
                    <GameRenderer key={i} game={g}/>
                ))}</Slider>
            </div>}
        </div>
    )
}