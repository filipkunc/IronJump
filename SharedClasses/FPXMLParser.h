//
//  FPXMLParser.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 9/15/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPGameProtocols.h"

@class FPXMLParser;

@protocol FPXMLParserDelegate <NSObject>

- (void)parser:(FPXMLParser *)parser foundObject:(id<FPGameObject>)gameObject;

@end    

@interface FPXMLParser : NSObject <NSXMLParserDelegate>
{
    id<FPXMLParserDelegate> delegate;
    id<FPGameObject> currentObject;    
    NSString *currentElement;
    int depth;
}

@property (readwrite, assign) id<FPXMLParserDelegate> delegate;

@end
