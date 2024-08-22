if [[ "$api_host" == "" ]];then
	api_host=http://localhost:8080
fi
function publish(){
	curl $api_host/publish -X POST -H 'Content-Type:application/json' -d '{"topic":"test1","content":"good evening,sir"}'
}
function readLatest(){
	curl $api_host/read\?topic\=test1 -X GET
}
for va in "$@";do
	$va
done
