# PASM OOP Example — NASM target
# class Greeter → Greeter_greet label in .text

@arch('x86_64')
@format('win64')

@section('.text')

class Greeter:
    @export
    def greet(self):
        print("Hello from Greeter class!\n")
        exit(0)

@section('.text')
@export
def main():
    call(Greeter_greet)
