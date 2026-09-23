# Fractal explorer support

The Burning Ship and Tricorn projects share this small support library. `Fractal.Camera`
contains viewport geometry; `Fractal.Explorer` owns the synchronous terminal event loop,
reset behavior, keyboard/mouse navigation, redraws, and terminal shutdown. Each program
supplies its initial camera, renderer, and palette labels. One explorer runs at a time.

Run either consuming project; this library is not a standalone program.
