Unser Frontend ruft einen Dienst auf, der gelegentlich kurz ausfällt, und der Benutzer soll davon möglichst nichts merken. Eine asynchrone Operation soll bei Misserfolg mehrmals neu versucht werden, mit zunehmendem Abstand zwischen den Versuchen, und erst am Ende den Fehler nach oben geben.

Sprache: TypeScript
