# This is a guess the number game.
import random
secretNumber = random.randint(1, 20)
print('I am thinking of number between 1 and 20.')

#Ask the player to guess 6 times
for guess in range(1, 7):
    print('Take a guess')
    guess = int(input())

    if guess < secretNumber:
        print('Your guess is too low.')
    elif guess > secretNumber:
        print('Your guess is too high.')
    else:
        break #The condition is the correct guess!
if guess == secretNumber:
    print('Good job! You guessed my number in ' + str(guess) + ' guesses!')
else:
    print('Nope. the number I was thinking of was ' + str(guess))
