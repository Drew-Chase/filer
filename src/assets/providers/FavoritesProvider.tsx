import {createContext, Dispatch, ReactNode, SetStateAction, useCallback, useContext, useEffect, useState} from "react";

export type FavoriteItem = {
    name: string;
    path: string;
}

interface FavoritesContextType
{
    favorites: FavoriteItem[];
    addFavorite: (path: FavoriteItem) => void;
    removeFavorite: (path: string) => void;
    clearFavorites: () => void;
    isFavorited: (path: string) => boolean;
    setFavorites: Dispatch<SetStateAction<FavoriteItem[]>>;
}

const FavoritesContext = createContext<FavoritesContextType | undefined>(undefined);

export function FavoritesProvider({children}: { children: ReactNode })
{
    const [favorites, setFavorites] = useState<FavoriteItem[]>(JSON.parse(localStorage.getItem("path-favorites") || "[]"));

    const addFavorite = useCallback((item: FavoriteItem) =>
    {
        const newFavorites = [...favorites];
        item.path = item.path.replace(/\/+$/, "");
        newFavorites.push(item);
        setFavorites(newFavorites);
    }, [favorites]);
    const removeFavorite = useCallback((path: string) =>
    {
        const newFavorites = [...favorites].filter(favorite => favorite.path.replace(/\/+$/, "") !== path.replace(/\/+$/, ""));
        setFavorites(newFavorites);
    }, [favorites]);
    const clearFavorites = useCallback(() =>
    {
        setFavorites([]);
    }, []);
    const isFavorited = useCallback((path: string) =>
    {
        return favorites.find(i => i.path.replace(/\/+$/, "") === path.replace(/\/+$/, "")) != undefined;
    }, [favorites]);

    useEffect(() =>
    {
        localStorage.setItem("path-favorites", JSON.stringify(favorites));
    }, [favorites]);

    return (
        <FavoritesContext.Provider value={{favorites, addFavorite, removeFavorite, clearFavorites, isFavorited, setFavorites}}>
            {children}
        </FavoritesContext.Provider>
    );
}

export function useFavorites(): FavoritesContextType
{
    const context = useContext(FavoritesContext);
    if (!context)
    {
        throw new Error("useFavorites must be used within a FavoritesProvider");
    }
    return context;
}