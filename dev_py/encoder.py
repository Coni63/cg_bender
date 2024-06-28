from collections import defaultdict

def find_substrings(s):
    substr_count = defaultdict(int)
    length = len(s)

    for start in range(length):
        for end in range(start + 2, length + 1):  # take substrings of length 2 or more
            substr = s[start:end]
            if not substr.isalpha():  # cannot compress if there are a Fucntion already
                continue
            substr_count[substr] += 1
    
    return substr_count

def gain(subset, occurrences, macro_len):
    # before = len(subset) * occurrences  # before we have 2 times the substring
    # after = len(subset) + 2             # add a semi-colon and a number
    return len(subset) * (-occurrences + 1) + 1 + macro_len

def compress_string(s: str, macro: list[str]) -> str:
    if len(macro) == 9:
        return ";".join([s] + macro)
    
    substr_count = find_substrings(s)
    
    # Remove substrings that appear only once
    substr_count = {k: v for k, v in substr_count.items() if v > 1}

    # Find the substring removing the most characters
    s_macro = str(len(macro))
    winner = ""
    g = 0
    for k, v in substr_count.items():
        _g = gain(k, v, len(s_macro))
        if _g < g:
            g = _g
            winner = k
    
    if winner == "":
        return ";".join([s] + macro)

    macro.append(winner)
    s = s.replace(winner, s_macro)

    return compress_string(s, macro)


if __name__ == "__main__":
    import time

    input_string = "DDDDRRDDDDDUUUUULLUUUULLLLUUDDRRRRDDDDRRDDDDDDDDDDDDLLUUUULLUUUUULURUULLLLRRRRDDLDRRRDDDDDLLLLUUUULRDDDDRRRRDDDDLLUDRRUUUULLUUUUULULLLLLUULLLLLLLLDDDDDDRRRRUDRRRRRRDLLDDDRRDDLLLLLLUUUULLLLUUUUUUUURRRRRRRRDRDRRRRRDRRDDDDDDDDDRRUUUUUUUUUUUULLUUUULLLLLLLLUUL"
    
    tic = time.time()
    compressed_output = compress_string(input_string, [])
    toc = time.time()
    print(compressed_output)
    print(toc - tic)