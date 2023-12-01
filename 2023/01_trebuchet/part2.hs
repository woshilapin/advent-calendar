import System.IO
import Data.Char

replaceNumber :: [Char] -> [Char]
replaceNumber [] = []
-- replaceNumber ('z':'e':'r':'o':rest) = '0':(replaceNumber ('o':rest)) -- may collapse with 'one'
replaceNumber ('o':'n':'e':rest) = '1':(replaceNumber ('e':rest)) -- may collapse with 'eight'
replaceNumber ('t':'w':'o':rest) = '2':(replaceNumber ('o':rest)) -- may collapse with 'one'
replaceNumber ('t':'h':'r':'e':'e':rest) = '3':(replaceNumber ('e':rest)) -- may collapse with 'eight'
replaceNumber ('f':'o':'u':'r':rest) = '4':(replaceNumber rest)
replaceNumber ('f':'i':'v':'e':rest) = '5':(replaceNumber ('e':rest)) -- may collapse with 'eight'
replaceNumber ('s':'i':'x':rest) = '6':(replaceNumber rest)
replaceNumber ('s':'e':'v':'e':'n':rest) = '7':(replaceNumber ('n':rest)) -- may collapse with 'nine'
replaceNumber ('e':'i':'g':'h':'t':rest) = '8':(replaceNumber ('t':rest)) -- may collapse with 'two' or 'three'
replaceNumber ('n':'i':'n':'e':rest) = '9':(replaceNumber ('e':rest)) -- may collapse with 'eight'
replaceNumber (c:rest) = c:(replaceNumber rest)

filterDigit :: [Char] -> [Int]
filterDigit s = [ digitToInt c | c <- s, isDigit c ]

firstLastAsNumber :: [Int] -> Int
firstLastAsNumber numbers = (head numbers) * 10 + (last numbers)

lineAsNumber :: IO Int
lineAsNumber = do
  line <- getLine
  return (firstLastAsNumber (filterDigit (replaceNumber line)))

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
