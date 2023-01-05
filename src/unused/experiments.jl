

function move_num!(numbers_picked)
    n = length(numbers_picked)
    seen = 0
    for i in n:-1:1
        if i == 1
            numbers_picked .= falses(n)
            return numbers_picked
        elseif numbers_picked[i] && !numbers_picked[i-1]
            numbers_picked[i-1] = true
            numbers_picked[end-seen+1:end] .= true
            numbers_picked[i:end-seen] .= false
            return numbers_picked
        elseif numbers_picked[i]
            seen += 1
        end
    end
end

payouts = [0, 0, 0, 0, 0, 10000, 36, 720, 360, 80, 252, 108, 72, 54, 180, 72, 180, 119, 36, 306, 1080, 144, 1800, 3600]

numbers_picked = falses(9)
numbers_picked[end-4:end] .= true
for j in 1:10000
    nums = (1:9)[.!numbers_picked]
    choices = (1:9)[numbers_picked]
    for k in nums
        sub_mask = falses(5)
        sub_mask[end-1:end] .= true
        set_payouts = Set{Int}()
        for _ in 1:20
            i = k + sum(choices[sub_mask])
            push!(set_payouts, payouts[i])
            move_num!(sub_mask)
            if sub_mask == falses(5)
                if length(set_payouts) == 10
                    println()
                    println(k)
                    println(choices)
                    println(set_payouts)
                else
                    #println(k, "\t", choices, "\t  okay\t", length(set_payouts), "\t", set_payouts)
                end
                break
            end
        end
    end

    #println(move_num!(numbers_picked))
    move_num!(numbers_picked)
    if numbers_picked == falses(9)
        println(j)
        break
    end
end

# results: there are 5 combinations that can end up with 10 distinct payout options,
# and at least one of them can actually come up during optimal play.
# so, I need to cover there being 10 options for results.
# Probably going to just replace the "reset" button in cases where there are 10 results.

function simplify(board::MMatrix{3, 3})
    b = board
    sequence = Function[]
    if (m = maximum(b[SA[2, 4, 6, 8]])) != 0
        while b[2] != m
            b = rotate(b)
            pushfirst!(sequence, unrotate)
        end
        if b[6] > b[4] || (b[4] == 0 && b[3] > b[1]) || (b[1] == 0 && b[7] > b[9])
            b = reflect(b)
            pushfirst!(sequence, unreflect)
        end
    elseif (m = maximum(b[SA[1, 3, 7, 9]])) != 0
        while b[1] != m
            b = rotate(b)
            pushfirst!(sequence, unrotate)
        end
        if b[7] > b[3] || (b[3] == 0 && b[4] > b[2]) || (b[2] == 0 && b[8] > b[6])
            b = rotate(reflect(b))
            pushfirst!(sequence, unreflect)
            pushfirst!(sequence, unrotate)
        end
    end
    return b, sequence
end

nothing

return [j == n ? T(cld(n*i, f)) : (one(T):n)[one(T):n .= T(cld(n*i, f))][perms[mod1(i, l), j]] for i in one(T):f, j in one(T):n]