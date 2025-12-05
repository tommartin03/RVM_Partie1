BRed='\033[1;31m'         # Red
BGreen='\033[1;32m'       # Green
Off='\033[0m'       # Text Reset

last_path=$(pwd)
examples_path="../examples2" # insert global path to examples

if [ -n "$1" ]
  then
    cd "$1"
fi



RVM='rvm'

# export GLOBIGNORE="examples2/ok/_*.tasm"
tasm_files=$(ls ${examples_path}/ok/*.tasm -v)
# unset GLOBIGNORE

cargo build --release

if [ $? -ne 0 ] ; then
    echo ""
    echo -e "${BRed}rvm failed to compile...${Off}"
    echo ""
    exit 1
fi

echo ""


for file in $tasm_files; do
    path_no_ext="${file%.*}"

    echo "-------------------------------------------------------------"

    echo ""
    echo "Running '${path_no_ext}' tasm files:"
    echo ""
    RUST_LOG="" $RVM exec  ${path_no_ext}.tasm > ${path_no_ext}_gen.res 2> ${path_no_ext}_tmp.err
    if [ $? -ne 0 ] ; then
        echo ""
        cat ${path_no_ext}_tmp.err
        rm ${path_no_ext}_tmp.err
        echo ""
        echo -e "${BRed}rvm command failed, continuing...${Off}"
        echo ""
        continue
    fi
    rm ${path_no_ext}_tmp.err
    
    echo -n "Classic version: "
    if !(cmp --silent -- ${path_no_ext}.res  ${path_no_ext}_gen.res) ; then
        echo -e "${BRed}Unexpected result${Off}"
    else
        echo -e "${BGreen}Valid Result${Off}"
    fi

    if [ -e "${path_no_ext}.ctasm" ]; then
        RUST_LOG="" $RVM exec  ${path_no_ext}.ctasm > ${path_no_ext}_gen.cres 2> ${path_no_ext}_tmp.err

        if [ $? -ne 0 ] ; then
            echo ""
            cat ${path_no_ext}_tmp.err
            rm ${path_no_ext}_tmp.err
            echo ""
            echo -e "${BRed}rvm command failed on compact text assembly, continuing...${Off}"
            echo ""
            continue
        fi
        rm ${path_no_ext}_tmp.err
        
        echo -n "Compact version: "
        if !(cmp --silent -- ${path_no_ext}.res  ${path_no_ext}_gen.cres) ; then
            echo -e "${BRed}Unexpected result${Off}"
        else
            echo -e "${BGreen}Valid Result${Off}"
        fi
    else
        echo "No other version"
    fi

    echo ""
done

cd $last_path
