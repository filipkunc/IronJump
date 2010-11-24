//
//  FPXMLParser.m
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 9/15/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPXMLParser.h"

@implementation FPXMLParser

@synthesize delegate;

- (id)init
{
    self = [super init];
    if (self) 
    {
        currentObject = nil;
        currentElement = nil;
        depth = 0;
        delegate = nil;
    }
    return self;
}

- (void)dealloc 
{
    [currentObject release];
    [currentElement release];
    [super dealloc];
}

- (void)parser:(NSXMLParser *)parser didStartElement:(NSString *)elementName namespaceURI:(NSString *)namespaceURI
 qualifiedName:(NSString *)qName attributes:(NSDictionary *)attributeDict
{
    depth++;
    
    if (depth == 2)
    {
        Class currentClass = NSClassFromString(elementName);
        currentObject = [[currentClass alloc] init];
        
        [[currentClass performSelector:@selector(alloc)] init];
    }
    
    currentElement = elementName;
    [currentElement retain];
}

- (void)parser:(NSXMLParser *)parser foundCharacters:(NSString *)string
{
    if (depth == 3)
        [currentObject parseXMLElement:currentElement value:string];
}

- (void)parser:(NSXMLParser *)parser didEndElement:(NSString *)elementName namespaceURI:(NSString *)namespaceURI
 qualifiedName:(NSString *)qName
{
    if (depth == 2)
        [delegate parser:self foundObject:currentObject];
    
    [currentElement release];
    currentElement = nil;
    
    depth--;
}


@end
