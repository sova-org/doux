# [ ] Factoriser

Beaucoup de duplication de code. Une structure unifiée pour chaque paramètre :
- une grosse structure params avec une mini doc, etc, etc. Tout le monde la consomme.

# [ ] Audio input qui consomme trop vite car il est en mono

# [ ] Sova peut manger la documentation de Doux 

Ajouter une documentation similaire à celle de Hydra mais pour les paramètres du moteur Doux

# [ ] Gros changement

- un double thread, un thread qui ne fait pas grand chose, qui exécute le schéma
- un thread qui prépare le schéma et qui l'upload 

# [ ] Gate

Durée de tout sauf le release

# [ ] DADHSR Envelope

1) virer time
3) virer duration
4) ajouter envdelay (delay)
5) ajouter 'hold' (maintien à l'amplitude maximale)
6) on ne garde que gate : gate n'est plus un booléen, c'est une durée qui correspond à envdelay + attack + hold + decay + temps de sustain. release est un temps en plus après le relâchement de la gate.
7) pour le paramètre "voice", gate 0 est une voix infinie.

# [ ] Bytebeat

AJouter /doux/sound/bytebeat/formula/"LSMJDQLKDSJQSMJ"

# [ ] Envelopes ADSR comme modulations audio

- ad (côté langage)
- adr (côté langage)
- adsr (utiliser comme modèle)

# [ ] Faire sauter les enveloppes hard codées

- FM
- lowpass / highpass / bandpass (SVF)
- lowpass / highpass / bandpass (ladder)

# [ ] Faire sauter les LFO hard codés

- fblfo

# [ ] Vumètres (vecteur de u8 pour tout les canaux)

# [ ] Buffer / Block Size

BLOCK_SIZE pose problème
Virer Plaits
