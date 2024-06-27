from collections import defaultdict, Counter

def find_substrings(s):
    substr_count = defaultdict(int)
    length = len(s)

    for start in range(length):
        for end in range(start + 2, length + 1):  # take substrings of length 2 or more
            substr = s[start:end]
            if not all(c in "UDLR" for c in substr):  # cannot compress if there are a Fucntion already
                continue
            substr_count[substr] += 1
    
    return substr_count

def gain(subset, occurrences):
    before = len(subset) * occurrences
    after = len(subset) + 2  # add a semi-colon and a number
    return after - before

def compress_string(s: str, macro: list[str]) -> str:
    if len(macro) == 10:
        return ";".join([s] + macro)

    substr_count = find_substrings(s)
    
    # Remove substrings that appear only once
    substr_count = {k: v for k, v in substr_count.items() if v > 1}

    # Find the substring removing the most characters
    winner = ""
    g = 0
    for k, v in substr_count.items():
        _g = gain(k, v)
        if _g < g:
            g = _g
            winner = k
    
    if winner == "":
        return ";".join([s] + macro)

    macro.append(winner)
    s = s.replace(winner, str(len(macro)))

    return compress_string(s, macro)


if __name__ == "__main__":
    input_string = "DDDDRRDDDDDUUUUULLUUUULLLLUUDDRRRRDDDDRRDDDDDDDDDDDDLLUUUULLUUUUULURUULLLLRRRRDDLDRRRDDDDDLLLLUUUULRDDDDRRRRDDDDLLUDRRUUUULLUUUUULULLLLLUULLLLLLLLDDDDDDRRRRUDRRRRRRDLLDDDRRDDLLLLLLUUUULLLLUUUUUUUURRRRRRRRDRDRRRRRDRRDDDDDDDDDRRUUUUUUUUUUUULLUUUULLLLLLLLUUL"
    compressed_output = compress_string(input_string, [])
    print(compressed_output)