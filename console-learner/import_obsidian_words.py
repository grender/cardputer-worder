#!/usr/bin/env python3
"""Convert Obsidian card format to WordCard JSON format"""
import json
import re
from datetime import datetime

def parse_obsidian_entry(lines, start_idx):
    """Parse a single Obsidian card entry"""
    i = start_idx
    
    # Get the front (word)
    if not lines[i].strip().startswith('- ') or '#card #word' not in lines[i]:
        return None, start_idx + 1
    
    front = lines[i].split('#')[0].replace('- ', '').strip()
    i += 1
    
    # Parse metadata
    metadata = {}
    while i < len(lines):
        line = lines[i]
        # Check if line starts with spaces followed by "card-"
        stripped = line.strip()
        if not stripped.startswith('card-'):
            break
        
        # Parse metadata line
        if '::' in line:
            # Keep original to check indentation
            if line.startswith('  ') or line.startswith('\t'):
                key, value = stripped.split('::', 1)
                metadata[key.strip()] = value.strip()
        i += 1
    
    # Get back translation and examples
    back = ""
    examples = []
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        
        # Stop if we hit a new card
        if stripped.startswith('-') and '#card #word' in stripped:
            break  # New card detected
        
        if not stripped or stripped == '---':
            if not back:
                i += 1
                continue
            else:
                break
        
        # Check for lines starting with "- " (back translation)
        if stripped.startswith('- '):
            content = stripped[2:].strip()
            if not back:
                back = content
            else:
                examples.append(content)
        
        i += 1
    
    return {
        'front': front,
        'back': back,
        'examples': examples,
        'metadata': metadata
    }, i

def convert_to_word_card(entry, word_id):
    """Convert parsed entry to WordCard JSON format"""
    front = entry['front']
    back = entry['back']
    examples = entry['examples']
    metadata = entry['metadata']
    
    # Extract metadata
    reps = int(metadata.get('card-repeats', 1))
    last_review = metadata.get('card-last-reviewed', '')
    due_date = metadata.get('card-next-schedule', '')
    ease_factor = float(metadata.get('card-ease-factor', 2.5))
    last_interval = float(metadata.get('card-last-interval', -1))
    
    # Determine state
    if reps == 1:
        state = "New"
    elif reps < 4:
        state = "Learning"
    else:
        state = "Review"
    
    # Parse timestamps
    created_at = 0
    last_review_ts = None
    
    if last_review:
        try:
            # Parse ISO format with timezone
            dt_str = last_review
            if dt_str.endswith('Z'):
                dt_str = dt_str[:-1] + '+00:00'
            dt = datetime.fromisoformat(dt_str)
            created_at = int(dt.timestamp())
            last_review_ts = int(dt.timestamp())
        except Exception as e:
            print(f"Error parsing timestamp {last_review}: {e}")
            created_at = int(datetime.utcnow().timestamp())
    
    # Default due date
    if not due_date:
        due_date = datetime.utcnow().isoformat() + "Z"
    
    # Calculate lapses
    lapses = max(0, reps - 1) if reps > 1 else 0
    
    # Create card object
    # Convert elapsed_days and scheduled_days to integers
    elapsed = int(max(0, last_interval)) if last_interval > 0 else 0
    scheduled = int(max(0, last_interval)) if last_interval > 0 else 0
    
    card = {
        "due": due_date,
        "stability": ease_factor * 0.5,
        "difficulty": ease_factor,
        "elapsed_days": elapsed,
        "scheduled_days": scheduled,
        "reps": reps,
        "lapses": lapses,
        "state": state,
        "last_review": last_review if last_review else datetime.utcnow().isoformat() + "Z"
    }
    
    return {
        "id": word_id,
        "front": front,
        "back": back,
        "examples": examples,
        "card": card,
        "created_at": created_at if created_at else int(datetime.utcnow().timestamp()),
        "last_review": last_review_ts
    }

def main():
    # The Obsidian text from user
    obsidian_text = """- splendid #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.36
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:45:33.090Z
  card-last-score:: 1
	- великолепный
- compulsive #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.36
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:40.418Z
  card-last-score:: 1
	- непреодолимый
- feast #card #word
  card-last-interval:: 8.88
  card-repeats:: 3
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-15T06:43:45.619Z
  card-last-reviewed:: 2024-12-06T09:43:45.620Z
  card-last-score:: 3
	- пир, праздник
- pageant #card #word
  card-last-interval:: 5.96
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-19T09:48:00.957Z
  card-last-reviewed:: 2024-12-13T10:48:00.958Z
  card-last-score:: 3
	- действо, церемония, процессия
- unveiled #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-22T21:57:23.035Z
  card-last-reviewed:: 2024-12-07T20:57:23.035Z
  card-last-score:: 3
	- раскрывать, развуалировать
	-
- urged #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:45:47.133Z
  card-last-score:: 1
	- побуждение, стремление
	- urge to go back - потянуло вернуться назад
- quarrel #card #word
  card-last-interval:: 8.88
  card-repeats:: 3
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-17T20:06:46.389Z
  card-last-reviewed:: 2024-12-08T23:06:46.389Z
  card-last-score:: 3
	- ссоры
- wet nurse #card #word
  card-last-interval:: 23.43
  card-repeats:: 4
  card-ease-factor:: 2.42
  card-next-schedule:: 2025-01-03T04:08:09.762Z
  card-last-reviewed:: 2024-12-10T18:08:09.763Z
  card-last-score:: 3
	- кормилица
- make light of #card #word
  card-last-interval:: 17.31
  card-repeats:: 4
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-25T03:42:24.856Z
  card-last-reviewed:: 2024-12-07T20:42:24.856Z
  card-last-score:: 3
	- относиться не серьезно
- tension #card #word
  card-last-interval:: 17.31
  card-repeats:: 4
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-25T03:42:28.361Z
  card-last-reviewed:: 2024-12-07T20:42:28.361Z
  card-last-score:: 3
	- напряжение
- perilous #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:03.088Z
  card-last-score:: 1
	- опасно, рисковано
- unease #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:16.917Z
  card-last-score:: 1
	- неловкость
- bowels turned to water #card #word
  card-last-interval:: 19.01
  card-repeats:: 4
  card-ease-factor:: 2.18
  card-next-schedule:: 2024-12-28T08:40:26.738Z
  card-last-reviewed:: 2024-12-09T08:40:26.738Z
  card-last-score:: 3
	- внутренности стали водой (дословно)
	- максимально испугался
- raise hackles #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:07:32.640Z
  card-last-reviewed:: 2024-12-08T23:07:32.641Z
  card-last-score:: 3
	- шерсть дыбом
- rustle  #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:47:50.823Z
  card-last-score:: 1
	- шелест/ шелестеть
- implacable  #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:11.787Z
  card-last-score:: 1
	- безжалостно
- hell bent #card #word
  card-last-interval:: 6.43
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-19T20:47:02.561Z
  card-last-reviewed:: 2024-12-13T10:47:02.562Z
  card-last-score:: 3
	- с дъявольским упорством
- heir  #card #word
  card-last-interval:: 17.31
  card-repeats:: 4
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-25T03:43:26.695Z
  card-last-reviewed:: 2024-12-07T20:43:26.695Z
  card-last-score:: 3
	- наследник
- slender  #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:07:28.606Z
  card-last-reviewed:: 2024-12-08T23:07:28.606Z
  card-last-score:: 3
	- тонкий / стройный
- graceful  #card #word
  card-last-interval:: 11.73
  card-repeats:: 4
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-20T16:07:50.954Z
  card-last-reviewed:: 2024-12-08T23:07:50.954Z
  card-last-score:: 3
	- грациозный/изящный
- destrier/garron  #card #word
  card-last-interval:: 28.3
  card-repeats:: 4
  card-ease-factor:: 2.66
  card-next-schedule:: 2025-01-08T01:08:20.363Z
  card-last-reviewed:: 2024-12-10T18:08:20.364Z
  card-last-score:: 5
	- вид коней подходящий для рыцарей- крупные [Дестриэ](https://ru.wikipedia.org/wiki/%D0%94%D0%B5%D1%81%D1%82%D1%80%D0%B8%D1%8D)
	- маленькие кони-пони но крепкие [Garron](https://en.wikipedia.org/wiki/Garron)
- vocation #card #word
  card-last-interval:: 23.43
  card-repeats:: 4
  card-ease-factor:: 2.42
  card-next-schedule:: 2025-01-03T04:08:13.812Z
  card-last-reviewed:: 2024-12-10T18:08:13.812Z
  card-last-score:: 3
	- призвание
- ring mail #card #word
  card-last-interval:: 19.01
  card-repeats:: 4
  card-ease-factor:: 2.18
  card-next-schedule:: 2024-12-27T23:07:00.851Z
  card-last-reviewed:: 2024-12-08T23:07:00.852Z
  card-last-score:: 5
	- кольчуга
- sable #card #word
  card-last-interval:: 8.88
  card-repeats:: 3
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-16T17:43:31.172Z
  card-last-reviewed:: 2024-12-07T20:43:31.172Z
  card-last-score:: 3
	- соболиный
- fortnight #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:07:23.812Z
  card-last-reviewed:: 2024-12-08T23:07:23.812Z
  card-last-score:: 3
	- две недели
- lordling #card #word
  card-last-interval:: 19.01
  card-repeats:: 4
  card-ease-factor:: 2.18
  card-next-schedule:: 2024-12-28T08:40:03.672Z
  card-last-reviewed:: 2024-12-09T08:40:03.673Z
  card-last-score:: 3
	- молодой лорд
- poacher #card #word
  card-last-interval:: 19.01
  card-repeats:: 4
  card-ease-factor:: 2.18
  card-next-schedule:: 2024-12-28T08:40:17.323Z
  card-last-reviewed:: 2024-12-09T08:40:17.323Z
  card-last-score:: 3
	- браконьер
- buck #card #word
  card-last-interval:: 23.43
  card-repeats:: 4
  card-ease-factor:: 2.42
  card-next-schedule:: 2025-01-01T18:40:20.345Z
  card-last-reviewed:: 2024-12-09T08:40:20.346Z
  card-last-score:: 5
	- олень
- ridge #card #word
  card-last-interval:: 8.72
  card-repeats:: 3
  card-ease-factor:: 2.18
  card-next-schedule:: 2024-12-22T03:47:44.876Z
  card-last-reviewed:: 2024-12-13T10:47:44.877Z
  card-last-score:: 5
	- гребень горный
- admit #card #word
  card-last-interval:: 23.43
  card-repeats:: 4
  card-ease-factor:: 2.42
  card-next-schedule:: 2025-01-01T18:40:30.446Z
  card-last-reviewed:: 2024-12-09T08:40:30.446Z
  card-last-score:: 5
	- признавать
- cruel #card #word
  card-last-interval:: 17.31
  card-repeats:: 4
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T04:11:22.780Z
  card-last-reviewed:: 2024-11-28T21:11:22.780Z
  card-last-score:: 3
	- жестокий
- shrugged #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:07:36.338Z
  card-last-reviewed:: 2024-12-08T23:07:36.339Z
  card-last-score:: 3
	- пожимать плечами
- despite #card #word
  card-last-interval:: 8.88
  card-repeats:: 3
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-22T12:41:01.620Z
  card-last-reviewed:: 2024-12-13T15:41:01.622Z
  card-last-score:: 3
	- несмотря на
- shivered #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.36
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:45:51.938Z
  card-last-score:: 1
	- дрожь
- mutter #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T03:41:52.320Z
  card-last-reviewed:: 2024-12-07T20:41:52.322Z
  card-last-score:: 3
	- бормотание
- drowsy #card #word
  card-last-interval:: 5.96
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-19T09:48:06.386Z
  card-last-reviewed:: 2024-12-13T10:48:06.387Z
  card-last-score:: 3
	- сонный
- eloquence #card #word
  card-last-interval:: 4
  card-repeats:: 2
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-17T10:45:23.247Z
  card-last-reviewed:: 2024-12-13T10:45:23.248Z
  card-last-score:: 3
	- красноречие
- stump #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:06:50.210Z
  card-last-reviewed:: 2024-12-08T23:06:50.211Z
  card-last-score:: 3
	- обрубок/культя
- hunched #card #word
  card-last-interval:: 4
  card-repeats:: 2
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-14T18:07:17.792Z
  card-last-reviewed:: 2024-12-10T18:07:17.798Z
  card-last-score:: 3
	- ссутулился
- sullen #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:23.653Z
  card-last-score:: 1
	- мрачный/угрюмый
- casually #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:47:40.813Z
  card-last-score:: 1
	- мимоходом, ненароком
- certainty #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:43.342Z
  card-last-score:: 1
	- уверенность
- weeping #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T03:41:39.426Z
  card-last-reviewed:: 2024-12-07T20:41:39.427Z
  card-last-score:: 3
	- Покрытый влагой, мокрый
- nod #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:07:39.497Z
  card-last-reviewed:: 2024-12-08T23:07:39.498Z
  card-last-score:: 3
	- Кивок
- lad #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:07:43.181Z
  card-last-reviewed:: 2024-12-08T23:07:43.181Z
  card-last-score:: 3
	- Юноша
- Cocksure #card #word
  card-last-interval:: 15.05
  card-repeats:: 4
  card-ease-factor:: 1.94
  card-next-schedule:: 2024-12-24T00:07:46.898Z
  card-last-reviewed:: 2024-12-08T23:07:46.898Z
  card-last-score:: 3
	- Самоуверенный
- Insolent #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.36
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:28.413Z
  card-last-score:: 1
	- Дерзкий, наглый, нахальный
- Deign #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.36
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:52.191Z
  card-last-score:: 1
	- Соизволить, снизойти,удостоить
- Scabbard #card #word
  card-last-interval:: 9.28
  card-repeats:: 3
  card-ease-factor:: 2.32
  card-next-schedule:: 2024-12-17T02:42:01.166Z
  card-last-reviewed:: 2024-12-07T20:42:01.166Z
  card-last-score:: 3
	- Ножны
- acquiescence #card #word
  card-last-interval:: 8.88
  card-repeats:: 3
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-18T05:39:52.689Z
  card-last-reviewed:: 2024-12-09T08:39:52.690Z
  card-last-score:: 3
	- Уступка, согласие
- thicket #card #word
  card-last-interval:: 8.88
  card-repeats:: 3
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-20T05:28:31.531Z
  card-last-reviewed:: 2024-12-11T08:28:31.532Z
  card-last-score:: 3
	- Чаща, заросли,  кустарник
- Vantage #card #word
  card-last-interval:: 11.73
  card-repeats:: 4
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-20T16:07:54.576Z
  card-last-reviewed:: 2024-12-08T23:07:54.577Z
  card-last-score:: 3
	- преимущество
- Sentinel #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-17T15:39:58.812Z
  card-last-reviewed:: 2024-12-09T08:39:58.813Z
  card-last-score:: 3
	- часовой
- Crust #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T03:42:07.941Z
  card-last-reviewed:: 2024-12-07T20:42:07.941Z
  card-last-score:: 3
	- корка
- Slither #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:45:55.696Z
  card-last-score:: 1
	- скользить
- Tug #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T03:42:04.901Z
  card-last-reviewed:: 2024-12-07T20:42:04.902Z
  card-last-score:: 3
	- рывок, дергать
- Dare #card #word
  card-last-interval:: 17.31
  card-repeats:: 4
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T04:11:26.440Z
  card-last-reviewed:: 2024-11-28T21:11:26.440Z
  card-last-score:: 3
	- вызов
- Abandoned #card #word
  card-last-interval:: 8.88
  card-repeats:: 3
  card-ease-factor:: 2.22
  card-next-schedule:: 2024-12-16T20:56:44.426Z
  card-last-reviewed:: 2024-12-07T23:56:44.426Z
  card-last-score:: 3
	- покинутый/брошенный
- Grope #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:54.854Z
  card-last-score:: 1
  collapsed:: true
	- идти на ощупь
- Sweep #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T03:42:15.832Z
  card-last-reviewed:: 2024-12-07T20:42:15.833Z
  card-last-score:: 3
	- мести, подметать
- Reluctantly #card #word
  card-last-interval:: -1
  card-repeats:: 1
  card-ease-factor:: 2.5
  card-next-schedule:: 2024-12-13T21:00:00.000Z
  card-last-reviewed:: 2024-12-13T10:46:19.760Z
  card-last-score:: 1
	- нежелание
- Sap #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T03:42:20.125Z
  card-last-reviewed:: 2024-12-07T20:42:20.125Z
  card-last-score:: 3
	- соки дерева
- Slipped #card #word
  card-last-interval:: 10.24
  card-repeats:: 3
  card-ease-factor:: 2.56
  card-next-schedule:: 2024-12-18T01:41:56.665Z
  card-last-reviewed:: 2024-12-07T20:41:56.665Z
  card-last-score:: 3
	- скользнуть
- Dirk #card #word
  card-last-interval:: 8.32
  card-repeats:: 3
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T03:42:12.397Z
  card-last-reviewed:: 2024-12-07T20:42:12.398Z
  card-last-score:: 3
	- кинжал
- Sheath #card #word
  card-last-interval:: 3.09
  card-repeats:: 2
  card-ease-factor:: 2.08
  card-next-schedule:: 2024-12-16T12:45:42.717Z
  card-last-reviewed:: 2024-12-13T10:45:42.717Z
  card-last-score:: 3
	- чехол/ножны
-"""
    
    lines = obsidian_text.split('\n')
    entries = []
    i = 0
    
    print(f"Total lines: {len(lines)}")
    print(f"First few lines: {lines[:3]}")
    
    entries_count = 0
    skip_count = 0
    while i < len(lines):
        entry, new_idx = parse_obsidian_entry(lines, i)
        if entry:
            entries.append(entry)
            entries_count += 1
            print(f"Parsed entry {entries_count}: {entry['front']} (i={i}->{new_idx})")
        else:
            skip_count += 1
            if skip_count < 10:  # Only print first few skips
                print(f"  Skipped line {i}: '{lines[i][:50]}'")
        if new_idx <= i:  # Safety check to prevent infinite loop
            i += 1
        else:
            i = new_idx
    
    # Load existing words
    try:
        with open('words.json', 'r', encoding='utf-8') as f:
            existing_words = json.load(f)
    except:
        existing_words = []
    
    # Convert entries to word cards
    start_id = len(existing_words) + 1
    new_words = []
    
    print(f"Found {len(entries)} entries to process")
    for idx, entry in enumerate(entries):
        if not entry['back']:
            print(f"Warning: Entry '{entry['front']}' has no back translation")
        # Debug: Show metadata for first few entries
        if idx < 3:
            print(f"Entry {idx+1} metadata: {entry['metadata']}")
        word_card = convert_to_word_card(entry, start_id + idx)
        # Check if this word already exists by front
        if not any(w.get('front') == word_card['front'] for w in existing_words):
            new_words.append(word_card)
    
    # Merge
    all_words = existing_words + new_words
    
    # Save
    with open('words.json', 'w', encoding='utf-8') as f:
        json.dump(all_words, f, ensure_ascii=False, indent=2)
    
    print(f"Successfully imported {len(new_words)} new words")
    print(f"Total words in database: {len(all_words)}")

if __name__ == "__main__":
    main()

