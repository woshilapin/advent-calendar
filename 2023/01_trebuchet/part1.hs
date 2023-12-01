import System.IO
import Data.Char

filterDigit :: [Char] -> [Int]
filterDigit s = [ digitToInt c | c <- s, isDigit c ]

firstLastAsNumber :: [Int] -> Int
firstLastAsNumber numbers = (head numbers) * 10 + (last numbers)

lineAsNumber :: IO Int
lineAsNumber = do
  line <- getLine
  return (firstLastAsNumber (filterDigit line))

linesAsNumbers :: IO [Int]
linesAsNumbers = do
  done <- isEOF
  if done
    then return []
    else do
      number <- lineAsNumber
      numbers <- linesAsNumbers
      return (number:numbers)

main = do
  numbers <- linesAsNumbers
  let
    res = sum numbers
  print res
