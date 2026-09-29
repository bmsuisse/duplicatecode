Unser Dashboard reagiert auf das Scrollen und Verändern der Fenstergrösse und wird dadurch sehr träge, weil die Verarbeitung hunderte Male pro Sekunde läuft. Wir wollen eine Funktion so umhüllen, dass sie höchstens einmal in einem einstellbaren Zeitabstand tatsächlich ausgeführt wird, auch wenn sie ununterbrochen angefordert wird.

Sprache: TypeScript
