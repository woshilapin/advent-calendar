import System.IO
import qualified Data.Set as Set
import qualified Data.List as List

data Card = As | King | Queen | Joker | Ten | Nine | Eight | Seven | Six | Five | Four | Three | Two
  deriving (Eq, Ord, Show)

data HandType = FiveOfAKind | FourOfAKind | FullHouse | ThreeOfAKind | TwoPair | OnePair | HighCard
  deriving (Eq, Ord, Show)

data Hand = Hand { handType :: HandType, cards :: [Card],  bid :: Int }
  deriving (Eq, Ord, Show)

charToCard :: Char -> Card
charToCard 'A' = As
charToCard 'K' = King
charToCard 'Q' = Queen
charToCard 'J' = Joker
charToCard 'T' = Ten
charToCard '9' = Nine
charToCard '8' = Eight
charToCard '7' = Seven
charToCard '6' = Six
charToCard '5' = Five
charToCard '4' = Four
charToCard '3' = Three
charToCard '2' = Two

charsToCards :: [Char] -> [Card]
charsToCards [] = []
charsToCards (c:rest) = (charToCard c):(charsToCards rest)

handTypeFromCards :: [Card] -> HandType
handTypeFromCards hand = let
  s = Set.fromList hand
  in case (Set.size s) of
    1 -> FiveOfAKind
    5 -> HighCard
    4 -> OnePair
    3 -> let 
      orderedHand = List.sort hand
      in if (orderedHand !! 0) == (orderedHand !! 2)
        || (orderedHand !! 1) == (orderedHand !! 3)
        || (orderedHand !! 2) == (orderedHand !! 4)
        then ThreeOfAKind
        else TwoPair
    2 -> let 
      orderedHand = List.sort hand
      in if (orderedHand !! 0) == (orderedHand !! 3)
        || (orderedHand !! 1) == (orderedHand !! 4)
        then FourOfAKind
        else FullHouse

lineToHand :: IO Hand
lineToHand = do
  line <- getLine
  let 
    handAsChars = List.takeWhile (/= ' ') line
    cards = charsToCards handAsChars
    bidAsStr = List.drop 1 $ List.dropWhile (/= ' ') line
    bid = read bidAsStr :: Int
    handType = handTypeFromCards cards
    in return Hand { handType=handType, cards=cards, bid=bid }

readHands :: IO [Hand]
readHands = do
  done <- isEOF
  if not done
    then do
      hand <- lineToHand
      hands <- readHands
      return (hand:hands)
    else do
      return []

main = do
  hands <- readHands
  let orderedHands = List.reverse $ List.sort hands
  let total = List.sum $ List.map (\(rank, Hand { bid = bid }) -> rank * bid) $ List.zip [1..] orderedHands
  print total
