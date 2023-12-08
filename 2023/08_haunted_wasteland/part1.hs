import System.IO
import qualified Data.List as List
import qualified Data.Map as Map

data Instruction = L | R
  deriving (Show, Eq)

charToInstruction :: Char -> Instruction
charToInstruction 'L' = L
charToInstruction 'R' = R

lineToInstructions :: [Char] -> [Instruction]
lineToInstructions [] = []
lineToInstructions (c:rest) = (charToInstruction c):(lineToInstructions rest)

type NodeID = String
type Path = (NodeID, (NodeID, NodeID))
lineToPath :: String -> Path
lineToPath s = let
  source = List.takeWhile (/= ' ') s
  directionsStr = List.takeWhile (/= ')') $ List.drop 1 $ List.dropWhile (/= '(') s
  left = List.takeWhile (/= ',') directionsStr
  right = List.drop 2 $ List.dropWhile (/= ',') directionsStr
  in ( source, (left, right) )

readPaths :: IO [Path]
readPaths = do
  done <- isEOF
  if not done
    then do
      pathStr <- getLine
      let path = lineToPath pathStr
      paths <- readPaths
      return (path:paths)
    else do
      return []

walk :: Map.Map NodeID (NodeID, NodeID) -> NodeID -> [Instruction] -> [NodeID]
walk _ end@"ZZZ" _ = [end]
walk graph start (i:instructions) = let
  (left, right) = graph Map.! start
  next = case i of
    L -> left
    R -> right
  in start:(walk graph next instructions)

main = do
  line <- getLine
  let instructions = lineToInstructions line
  emptyLine <- getLine
  paths <- readPaths
  let graph = Map.fromList paths
  let way = walk graph "AAA" (List.cycle instructions)
  putStr "Arrived at destination in "
  putStr (show ((List.length way) - 1))
  putStr " steps\n"
