NEXA-3D-REF-001 — Canonical 3D Character Reference & Turnaround Specification v1.0

Specification ID: NEXA-3D-REF-001
System: Nexa AI Training Tutor
Version: 1.0
Status: Production Reference Baseline
Depends On: NEXA-CBS-001, NEXA-AVTR-001, NEXA-3D-001, NEXA-3D-ART-001

We are now at the point where the next output should become visual production material, not another long architectural detour.

1. Objective

Create a locked reference package from which the actual Blender model can be constructed.

The package SHALL establish one consistent character from every angle:

Nexa
├── Front
├── Front 3/4 Left
├── Front 3/4 Right
├── Left Profile
├── Right Profile
├── Back 3/4
├── Back
├── Facial Close-Up
├── Expression Sheet
├── Hand/Gesture Sheet
└── Material/Color Sheet

The key requirement is cross-view consistency. These cannot be independent interpretations of Nexa.

2. Canonical Character Direction

Nexa remains an adult female technical instructor with a stylized-realistic cyber/hacker aesthetic.

Her visual language should communicate:

intelligent • technically formidable • approachable • composed • slightly playful • modern • professional

She should look capable of teaching Rust ownership, debugging a kernel problem, explaining a TCP packet capture, or walking a learner through a cyber lab.

The design should avoid both extremes:

generic corporate assistant ←──── NEXA ────→ exaggerated cyberpunk caricature
3. Silhouette

The silhouette must remain recognizable even without textures.

Primary identity elements:

              ╭──────────╮
              │   HAIR   │
          ╭───┴──────────┴───╮
          │   distinctive    │
          │    glasses       │
          ╰──────┬─────┬─────╯
                 │face │
              ╭──┴─────┴──╮
             ╱ technical   ╲
            ╱    jacket     ╲
           │                 │
           │ athletic/slim   │
           │ adult build     │

Hair, glasses, face, and jacket are the strongest silhouette components.

4. Face

The face is the highest-priority modeling reference.

Target characteristics:

adult feminine proportions;
slightly stylized rather than photographic;
defined but not severe jaw;
expressive cheek structure;
medium-small nose;
highly readable eyebrows;
large but believable eyes;
expressive mouth suitable for speech;
face capable of appearing intelligent without defaulting to stern.

The neutral expression should already feel attentive.

5. Eyes

Eyes are one of Nexa's most important communication mechanisms.

They should be:

slightly larger than strict realism
almond shaped
highly readable
strong iris definition
dark lashes
clear catchlight

They must work equally well for:

eye contact
thinking
skepticism
curiosity
concern
encouragement

The whites of the eyes should not be excessively bright.

6. Eyebrows

Brows should be highly expressive and relatively strong.

They are essential for Nexa's:

focused
curious
skeptical
corrective
surprised

states.

The neutral brow position must not make her appear angry.

7. Mouth

The mouth should be optimized visually for animation.

Reference imagery must clearly establish:

upper lip contour
lower lip volume
mouth width
corners
Cupid's bow
neutral closure
smile shape

Lip proportions must leave enough range for the canonical visemes.

8. Hair

Hair is a major identity feature.

Preferred direction:

dark brown / near-black base
medium-long
layered
strong silhouette
side/frontal framing
select loose strands
subtle purple/cool highlights

It should feel stylish and technical without looking costume-like.

6

The production model should favor larger controllable hair masses rather than hundreds of independent strands.

9. Glasses

The glasses remain a signature feature.

Direction:

thin technical frame
slightly futuristic
dark graphite/black
subtle violet or cyan technology detail
transparent lenses

They should look like something an advanced systems engineer might actually wear—not a giant sci-fi visor.

They also need enough physical structure to support Nexa's adjust-glasses gesture.

10. Body Proportions

Initial reference target:

Height              ~1.68 m
Head/body ratio     ~7.25 heads
Build               slim / naturally athletic
Shoulders           moderate
Torso               realistic
Waist                defined but natural
Leg length           realistic/slightly elongated
Hands                realistic, expressive

These become modeling references, not arbitrary runtime scaling.

11. Clothing

The primary outfit becomes the Nexa Technical Jacket.

Outer layer

Dark fitted technical jacket:

graphite / near-black
structured shoulders
clean silhouette
high-quality fabric
purple accent lines
small cyan technological details
Inner layer

Simple dark technical top.

The jacket should carry most of the visual complexity.

Lower body

Dark fitted technical trousers with restrained paneling.

Footwear

Dark boots or technical shoes.

No excessive armor.

12. Accent Language

Primary visual palette:

Role	Direction
Base	Graphite / near-black
Secondary	Dark charcoal
Primary accent	Electric violet
Secondary accent	Cool cyan
Skin	Natural warm-neutral
Hair	Near-black / dark brown
Metal	Gunmetal
Emission	Violet/cyan, restrained

The accent ratio should remain heavily weighted toward dark neutral materials.

13. Cyber Elements

Cyber styling should appear through details such as:

jacket seam illumination
small collar indicator
glasses electronics
wrist interface
subtle material panels

Not:

giant glowing armor
dozens of cables
constant neon
weaponry
oversized implants

She is a tutor and technical expert first.

14. Front Turnaround

The front reference SHALL establish:

head width
eye spacing
jaw width
shoulder width
torso length
arm length
waist
hip width
leg proportions
clothing symmetry

Pose:

A-pose
feet shoulder-width or slightly narrower
hands relaxed
neutral expression
camera orthographic

No dramatic perspective.

15. Side Reference

The side view must establish:

forehead
nose projection
lip projection
chin
jaw
ear
neck angle
chest
spinal curve
pelvis
knee
heel
hair volume

This is critical for avoiding the common problem where an attractive front view produces an incorrect 3D profile.

16. Back Reference

The back establishes:

hair volume
shoulder blades
jacket construction
waist
hip shape
rear garment seams
leg silhouette

The hair should not hide every useful modeling landmark.

17. 3/4 Reference

The 3/4 view is arguably the most important identity check.

Nexa will spend a large amount of tutor interaction near this orientation.

It should immediately read as the same character as the front view.

18. Facial Close-Up Sheet

Produce:

FRONT
3/4 LEFT
PROFILE

at substantially higher resolution than the body turnaround.

Include visible:

hairline
eyebrows
eyelids
iris
nose
lips
jaw
ears
glasses contact points
19. Expression Sheet

The first sheet SHALL contain:

┌──────────────┬──────────────┬──────────────┐
│   Neutral    │ Soft Smile   │  Encouraging │
├──────────────┼──────────────┼──────────────┤
│   Focused    │   Curious    │   Thinking   │
├──────────────┼──────────────┼──────────────┤
│  Skeptical   │  Concerned   │   Serious    │
├──────────────┼──────────────┼──────────────┤
│  Surprised   │  Confused    │  Corrective  │
└──────────────┴──────────────┴──────────────┘

All must use the same camera, head orientation, lighting, hair, glasses, and character proportions.

Only facial performance changes.

20. Critical Expression: Focused

This will probably become one of Nexa's most frequently used states.

It should combine:

slight brow compression
strong eye attention
neutral mouth
slight forward head orientation

It must not look angry.

21. Critical Expression: Thinking

Thinking should use:

slightly offset gaze
small brow asymmetry
subtle lip compression

rather than an exaggerated cartoon face.

22. Critical Expression: Skeptical

This is an important personality state.

Use:

single raised brow
slight head tilt
subtle asymmetric mouth

This can give Nexa considerable personality without dialogue.

23. Critical Expression: Encouraging

Avoid an enormous commercial-assistant smile.

Use:

warm eye engagement
soft smile
slightly raised cheeks
relaxed brows
24. Corrective Expression

When the learner makes an error:

focused eyes
slight brow contraction
neutral/slightly asymmetric mouth

It should communicate:

“That isn't quite right; let's inspect it.”

not judgment.

25. Viseme Reference Sheet

The modeling reference set should also include:

REST   A   E   I   O   U
MBP    FV  L   WQ  TH
CHSH   R

Front-facing close-up, identical camera.

These become direct references for Blender shape keys.

26. Hand Reference Sheet

Because Nexa will point constantly, hand references are mandatory.

Include:

relaxed
open palm
point
pinch
fist
thumbs up
typing
two-finger precision gesture

Views:

palm
back
side
3/4
27. Gesture Sheet

Full upper-body poses:

Point Left
Point Right
Open-Hand Explain
Two-Hand Explain
Thinking
Adjust Glasses
Attention
Thumbs Up
Typing
Listening

These become animation reference material.

28. Signature Tutor Pose

Nexa should have a recognizable default instructional pose:

upper body upright
shoulders relaxed
slight forward engagement
one hand near workstation
other hand available for gesture
head facing learner
direct but comfortable eye contact

This eventually becomes Idle_Seated.

29. Workstation Reference

The first environment reference should establish Nexa at her cyber workstation.

6

The workstation should use:

dark materials
purple/cyan practical lighting
large lesson display
terminal/code display
subtle holographic elements

but maintain strong visual hierarchy around Nexa.

30. Tutor Camera Reference

Primary desktop framing:

         HEADROOM
            │
      ┌─────────────┐
      │    HEAD     │
      │   SHOULDERS │
      │             │
      │    HANDS    │
      │             │
      └─────────────┘
          WAIST

The hands must remain visible enough for gestures.

31. Character Reference Lock Criteria

Before Blender production begins in earnest, the reference package SHALL pass:

Front and profile facial proportions agree.
3/4 view unmistakably matches the front.
Hair silhouette is consistent from every angle.
Glasses geometry is physically plausible.
Clothing seams align across views.
Body height/proportions agree.
Expressions preserve character identity.
Neutral expression reads as attentive.
Focused does not read as angry.
Encouraging does not become overly cheerful.
Skeptical remains subtle.
Hands are sufficiently detailed for instructional gestures.
Cyber elements remain restrained.
Nexa reads as a technical expert rather than a generic assistant.
32. Reference Package Structure
assets/
└── nexa/
    └── reference/
        ├── character/
        │   ├── front.png
        │   ├── three-quarter-left.png
        │   ├── three-quarter-right.png
        │   ├── profile-left.png
        │   ├── profile-right.png
        │   ├── back-three-quarter.png
        │   └── back.png
        │
        ├── face/
        │   ├── neutral-front.png
        │   ├── neutral-three-quarter.png
        │   └── neutral-profile.png
        │
        ├── expressions/
        │   └── expression-sheet.png
        │
        ├── visemes/
        │   └── viseme-sheet.png
        │
        ├── hands/
        │   └── hand-reference.png
        │
        ├── gestures/
        │   └── gesture-sheet.png
        │
        ├── materials/
        │   └── material-reference.png
        │
        └── environment/
            └── workstation-reference.png
33. We Have Reached the Visual Production Gate

This is the point where I recommend we actually generate the canonical turnaround instead of writing the next architecture document.

The next deliverable should be:

NEXA-3D-REF-002 — Canonical Character Turnaround

One production sheet containing:

FRONT
3/4 FRONT
SIDE
3/4 BACK
BACK

with Nexa in the same A-pose, same clothing, same proportions, same face, neutral lighting, orthographic-style presentation, and no environment.

After that:

REF-003 → facial close-ups
REF-004 → expression sheet
REF-005 → viseme sheet
REF-006 → hands
REF-007 → gestures

Then we have enough locked visual information to open Blender and start building the real Nexa mesh.

So the next step is the actual turnaround image.
