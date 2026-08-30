import re

with open("frontend/src/app/globals.css", "r") as f:
    content = f.read()

start_marker = r"/\* Horizontal Journey — layout & track styles \*/"
match = re.search(start_marker, content)
if match:
    # Remove from this marker down to the next comment block `/* Mobile Swipe Tracks: Scrollbar Hiding & Overflow Containment */`
    end_marker = r"/\* Mobile Swipe Tracks: Scrollbar Hiding & Overflow Containment \*/"
    end_match = re.search(end_marker, content)
    
    if end_match:
        new_content = content[:match.start()] + content[end_match.start():]
        with open("frontend/src/app/globals.css", "w") as f:
            f.write(new_content)
        print("CSS cleaned")
    else:
        print("Could not find end marker")
else:
    print("Could not find start marker")
