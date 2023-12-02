import System.IO
import Data.List

data Cube = Red Int | Green Int | Blue Int

parseGameID :: String -> Int
parseGameID s = read (drop 5 s) :: Int

parseColor :: [Char] -> (Int -> Cube)
parseColor ['r', 'e', 'd'] = Red
parseColor ['g', 'r', 'e', 'e', 'n'] = Green
parseColor ['b', 'l', 'u', 'e'] = Blue

parseCube :: String -> Cube
parseCube s =  let
  numberStr = takeWhile (/= ' ') s
  number = read numberStr :: Int
  colorStr = drop 1 $ dropWhile (/= ' ') s
  colorFn = parseColor colorStr
  in
    colorFn number

parseSet :: String -> [Cube]
parseSet "" = []
parseSet s = let
  cubeDraw = takeWhile (/= ',') s
  restDraw = drop 2 $ dropWhile (/= ',') s
  cube = parseCube cubeDraw
  restCube = parseSet restDraw
  in cube:restCube

parseBag :: String -> [[Cube]]
parseBag "" = []
parseBag s = let
  setStr = takeWhile (/= ';') s
  set = parseSet setStr
  restStr = drop 2 $ dropWhile (/= ';') s
  rest = parseBag restStr
  in set:rest

isValidCube :: Cube -> Bool
isValidCube (Red n) | n > 12 = False
isValidCube (Red n) = True
isValidCube (Green n) | n > 13 = False
isValidCube (Green n) = True
isValidCube (Blue n) | n > 14 = False
isValidCube (Blue n) = True

isValidSet :: [Cube] -> Bool
isValidSet [] = True
isValidSet (c:rest) = (isValidCube c) && (isValidSet rest)

isValidBag :: [[Cube]] -> Bool
isValidBag [] = True
isValidBag (s:rest) = (isValidSet s) && (isValidBag rest)

bagResult :: Int -> Bool -> Int
bagResult _ False = 0
bagResult n True = n

parseLine :: IO Int
parseLine = do
  line <- getLine
  let
    gameStr = takeWhile (/= ':') line
    gameID = parseGameID gameStr
    bagStr = drop 2 $ dropWhile (/= ':') line
    bag = parseBag bagStr
    bagValidity = isValidBag bag
    in return (bagResult gameID bagValidity)
  
parseLines :: IO Int
parseLines = do
  done <- isEOF
  if not done
    then do
      lineRes <- parseLine
      linesRes <- parseLines
      return (lineRes + linesRes)
    else
      return 0

main = do
  res <- parseLines
  print res
